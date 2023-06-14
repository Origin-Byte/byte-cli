// TODO: Add back minting with arguments, including mint_cap and warehouse
use crate::{
    consts::NFT_PROTOCOL,
    err::{self, RustSdkError},
    utils::MoveType,
};
use anyhow::Result;
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_crypto::intent::Intent;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use std::{thread, time};
use sui_json_rpc_types::SuiTransactionBlockEffects;
use sui_keys::keystore::AccountKeystore;
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    wallet_context::WalletContext,
};
use sui_types::{base_types::ObjectRef, parse_sui_type_tag};
use sui_types::{
    crypto::Signature, messages::TransactionData,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Serialize)]
pub struct NftData {
    pub name: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

impl NftData {
    pub fn to_map(&self) -> Result<Vec<SuiJsonValue>> {
        let mut params: Vec<SuiJsonValue> = Vec::new();

        if let Some(value) = &self.name {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.url {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.description {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(map) = &self.attributes {
            let keys: Vec<String> = map.clone().into_keys().collect();
            let values: Vec<String> = map.clone().into_values().collect();

            let keys_arr = json!(keys);
            let values_arr = json!(values);

            params.push(SuiJsonValue::new(keys_arr)?);
            params.push(SuiJsonValue::new(values_arr)?);
        }

        Ok(params)
    }
}

pub async fn create_warehouse(
    wallet_ctx: &WalletContext,
    // SuiAddress implements Copy
    sender: SuiAddress,
    collection_type: MoveType,
) -> Result<ObjectID, RustSdkError> {
    let package_id = ObjectID::from_str(NFT_PROTOCOL)
        .map_err(|err| err::object_id(err, NFT_PROTOCOL))?;

    let collection_type_ = collection_type.write_type();
    let module = Identifier::from_str("warehouse")?;
    let function = Identifier::from_str("init_warehouse")?;

    let mut builder = ProgrammableTransactionBuilder::new();
    let _res = builder.move_call(
        package_id,                                           // Package ID
        module,                                               // Module Name
        function,                                             // Function Name
        vec![parse_sui_type_tag(collection_type_.as_str())?], // Type Arguments
        vec![],                                               // Call Arguments
    );

    let data = TransactionData::new_programmable(
        sender,
        vec![], // Gas Objects
        builder.finish(),
        10_000, // Gas Budget
        1,      // Gas Price
    );

    // Sign transaction.
    let signatures: Vec<Signature> = vec![wallet_ctx
        .config
        .keystore
        .sign_secure(&sender, &data, Intent::sui_transaction())?];

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
        mint_nft(wallet_ctx, package_id, module_name, gas_budget, sender).await
    })
}

pub async fn mint_nft(
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
