// TODO: Add back minting with arguments, including mint_cap and warehouse
use crate::{
    err::{self, RustSdkError},
    metadata::Metadata,
    utils::{get_context, MoveType},
};
use anyhow::Result;
use move_core_types::identifier::Identifier;
use shared_crypto::intent::Intent;
use std::{collections::BTreeMap, str::FromStr, sync::Arc};
use std::{thread, time};
use sui_json_rpc_types::SuiExecutionStatus::Failure;
use sui_json_rpc_types::{SuiObjectDataOptions, SuiTransactionBlockEffects};
use sui_keys::keystore::AccountKeystore;
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    wallet_context::WalletContext,
};
use sui_types::{
    base_types::ObjectRef,
    messages::{CallArg, ObjectArg},
    parse_sui_type_tag,
};
use sui_types::{
    crypto::Signature, messages::TransactionData,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};
use tokio::task::JoinHandle;

struct MintEffects {
    minted_nfts: Vec<String>,
    error_logs: Vec<MintError>,
}

struct MintError {
    from_nft: u32,
    to_nft: u32,
    error: String,
}

pub enum MintEffect {
    Success(Vec<String>),
    Error(MintError),
}

pub async fn create_warehouse(
    collection_type: MoveType,
    package_id: ObjectID,
    gas_coin: ObjectRef,
) -> Result<ObjectID, RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();
    let keystore = &wallet_ctx.config.keystore;
    let sender = wallet_ctx.config.active_address.unwrap();

    let collection_type = collection_type.write_type();
    let module = Identifier::from_str("warehouse")?;
    let function = Identifier::from_str("init_warehouse")?;

    let mut builder = ProgrammableTransactionBuilder::new();
    let _res = builder.move_call(
        package_id,                                          // Package ID
        module,                                              // Module Name
        function,                                            // Function Name
        vec![parse_sui_type_tag(collection_type.as_str())?], // Type Arguments
        vec![],                                              // Call Arguments
    );

    let data = TransactionData::new_programmable(
        sender,
        vec![gas_coin], // Gas Objects
        builder.finish(),
        10_000_000, // Gas Budget
        1_000,      // Gas Price
    );

    // Sign transaction.
    let signatures: Vec<Signature> = vec![keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?];

    let response = wallet_ctx
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    let warehouse_id = effects.created.first().unwrap().reference.object_id;

    Ok(warehouse_id)
}

pub async fn handle_mint_nft(
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    sender: SuiAddress,
) -> JoinHandle<Result<Vec<ObjectID>, RustSdkError>> {
    tokio::spawn(async move {
        mint_nft_concurrent(
            wallet_ctx,
            package_id,
            module_name,
            gas_budget,
            sender,
        )
        .await
    })
}

pub async fn mint_nfts_in_batch(
    mut data: Vec<(u32, Metadata)>,
    wallet_ctx: &WalletContext,
    package_id: String,
    module_name: String,
    gas_budget: u64,
    sender: SuiAddress,
    warehouse: String,
    mint_cap: String,
) -> Result<MintEffect, RustSdkError> {
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;
    let warehouse_id = ObjectID::from_str(warehouse.as_str())
        .map_err(|err| err::object_id(err, warehouse.as_str()))?;
    let mint_cap_id = ObjectID::from_str(mint_cap.as_str())
        .map_err(|err| err::object_id(err, mint_cap.as_str()))?;

    let mut builder = ProgrammableTransactionBuilder::new();

    let objs = wallet_ctx
        .get_client()
        .await?
        .read_api()
        .multi_get_object_with_options(
            vec![mint_cap_id, warehouse_id],
            SuiObjectDataOptions::full_content(),
        )
        .await
        .unwrap();

    // Iterate over the entries and consume them
    while let Some((index, nft_data)) = data.pop() {
        let mut args = nft_data.into_args()?;
        objs.iter().for_each(|obj| {
            let obj_data = obj.data.as_ref().unwrap();
            let obj_ref: ObjectRef = (
                obj_data.object_id.clone(),
                obj_data.version,
                obj_data.digest,
            );
            args.push(CallArg::Object(ObjectArg::ImmOrOwnedObject(obj_ref)));
        });

        builder.move_call(
            package_id,
            Identifier::new(module_name.as_str()).unwrap(),
            Identifier::new("mint_nft_to_warehouse").unwrap(),
            vec![],
            args,
        )?;
    }

    // Get gas object
    let coins: Vec<ObjectRef> = wallet_ctx
        .gas_objects(sender)
        .await?
        .iter()
        // Ok to unwrap() since `get_gas_objects` guarantees gas
        .map(|(_val, object)| (object.object_id, object.version, object.digest))
        .collect();

    let pt = builder.finish();

    let data =
        TransactionData::new_programmable(sender, coins, pt, gas_budget, 1_000);

    // Sign transaction.
    let signatures: Vec<Signature> = vec![wallet_ctx
        .config
        .keystore
        .sign_secure(&sender, &data, Intent::sui_transaction())?];

    // Execute the transaction.
    println!("Executing transaction...");
    let response = wallet_ctx
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();
    println!("Effects: {:?}", effects);

    match effects.status {
        Success => {
            let nft_ids = effects.created;
            let mut i = 0;

            let nfts = nft_ids
                .iter()
                .map(|obj_ref| {
                    println!("NFT minted: {:?}", obj_ref.reference.object_id);
                    i += 1;
                    obj_ref.reference.object_id.to_string()
                })
                .collect::<Vec<String>>();

            return Ok(MintEffect::Success(nfts));
        }
        Failure { error } => {
            return Ok(MintEffect::Error(error));
        }
    }
}

pub async fn mint_nft_concurrent(
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    // SuiAddress implements Copy
    sender: SuiAddress,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;

    let mut retry = 0;
    let response = loop {
        let mut builder = ProgrammableTransactionBuilder::new();

        for _ in 0..1 {
            builder.move_call(
                package_id,
                Identifier::new(module_name.as_str()).unwrap(),
                Identifier::new("airdrop_nft").unwrap(),
                vec![],
                vec![],
            )?;
        }

        // Get gas object
        let coins: Vec<ObjectRef> = wallet_ctx
            .gas_objects(sender)
            .await?
            .iter()
            // Ok to unwrap() since `get_gas_objects` guarantees gas
            .map(|(_val, object)| {
                (object.object_id, object.version, object.digest)
            })
            .collect();

        let data = TransactionData::new_programmable(
            sender,
            coins, // Gas Objects
            builder.finish(),
            *gas_budget, // Gas Budget
            1_000,       // Gas Price
        );

        // Sign transaction.
        let signatures: Vec<Signature> = vec![wallet_ctx
            .config
            .keystore
            .sign_secure(&sender, &data, Intent::sui_transaction())?];

        // Execute the transaction.

        let response_ = wallet_ctx
            .execute_transaction_block(
                Transaction::from_data(
                    data,
                    Intent::sui_transaction(),
                    signatures,
                )
                .verify()?,
            )
            .await;

        if retry == 3 {
            break response_?;
        }

        if response_.is_err() {
            println!("Retrying mint...");
            let ten_millis = time::Duration::from_millis(1000);
            thread::sleep(ten_millis);
            retry += 1;
            continue;
        }
        break response_?;
    };

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    let nft_ids = effects.created;
    let mut i = 0;

    let nfts = nft_ids
        .iter()
        .map(|obj_ref| {
            println!("NFT minted: {:?}", obj_ref.reference.object_id);
            i += 1;
            obj_ref.reference.object_id
        })
        .collect::<Vec<ObjectID>>();

    Ok(nfts)
}

pub async fn handle_parallel_mint_nft(
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    gas_coin: Arc<ObjectRef>,
    sender: SuiAddress,
) -> JoinHandle<Result<Vec<ObjectID>, RustSdkError>> {
    tokio::spawn(async move {
        mint_nft_with_gas_coin(
            wallet_ctx,
            package_id,
            module_name,
            gas_budget,
            gas_coin,
            sender,
        )
        .await
    })
}

pub async fn mint_nft_with_gas_coin(
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    gas_coin: Arc<ObjectRef>,
    // SuiAddress implements Copy
    sender: SuiAddress,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;

    let mut retry = 0;
    let response = loop {
        let mut builder = ProgrammableTransactionBuilder::new();

        // TODO: generalise amount of NFTs minted
        for _ in 0..1_000 {
            builder.move_call(
                package_id,
                Identifier::new(module_name.as_str()).unwrap(),
                Identifier::new("airdrop_nft").unwrap(),
                vec![],
                vec![],
            )?;
        }

        let data = TransactionData::new_programmable(
            sender,
            vec![*gas_coin], // Gas Objects
            builder.finish(),
            *gas_budget, // Gas Budget
            1_000,       // Gas Price
        );

        // Sign transaction.
        let signatures: Vec<Signature> = vec![wallet_ctx
            .config
            .keystore
            .sign_secure(&sender, &data, Intent::sui_transaction())?];

        // Execute the transaction.
        let response_ = wallet_ctx
            .execute_transaction_block(
                Transaction::from_data(
                    data,
                    Intent::sui_transaction(),
                    signatures,
                )
                .verify()?,
            )
            .await;

        if retry == 3 {
            break response_?;
        }

        if response_.is_err() {
            println!("Retrying mint...");
            let ten_millis = time::Duration::from_millis(1000);
            thread::sleep(ten_millis);
            retry += 1;
            continue;
        }
        break response_?;
    };

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    let nft_ids = effects.created;
    let mut i = 0;

    let nfts = nft_ids
        .iter()
        .map(|obj_ref| {
            println!("NFT minted: {:?}", obj_ref.reference.object_id);
            i += 1;
            obj_ref.reference.object_id
        })
        .collect::<Vec<ObjectID>>();

    Ok(nfts)
}
