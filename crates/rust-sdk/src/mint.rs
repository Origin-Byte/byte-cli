use crate::{
    consts::NFT_PROTOCOL,
    err::{self, RustSdkError},
    utils::{
        get_active_address, get_client, get_context, get_keystore, MoveType,
    },
};
use anyhow::{anyhow, Result};
use gag::Gag;
use move_core_types::identifier::Identifier;
use serde::{Deserialize, Serialize};
use serde_json::json;
use shared_crypto::intent::Intent;
use std::{collections::HashMap, str::FromStr, sync::Arc};
use std::{thread, time};
use sui_json_rpc_types::{Coin, Page, SuiTransactionBlockEffects};
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    SuiClient,
};
use sui_types::{
    base_types::ObjectRef,
    coin,
    messages::{CallArg, ObjectArg, ProgrammableTransaction},
    parse_sui_type_tag, TypeTag, SUI_FRAMEWORK_OBJECT_ID,
};
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
    // Do we need this?
    _sui: &SuiClient,
    // keystore: &Keystore,
    // context: &mut WalletContext,
    // SuiAddress implements Copy
    sender: SuiAddress,
    collection_type: MoveType,
) -> Result<ObjectID, RustSdkError> {
    let context = get_context().await?;

    let package_id = ObjectID::from_str(NFT_PROTOCOL)
        .map_err(|err| err::object_id(err, NFT_PROTOCOL))?;

    let collection_type_ = collection_type.write_type();
    let module = Identifier::from_str("warehouse")?;
    let function = Identifier::from_str("init_warehouse")?;

    let mut builder = ProgrammableTransactionBuilder::new();
    let _res = builder.move_call(
        package_id, // Package ID
        module,     // Module Name
        function,   // Function Name
        vec![parse_sui_type_tag(collection_type_.as_str())?.into()], // Type Arguments
        vec![], // Call Arguments
    );

    let data = TransactionData::new_programmable(
        sender,
        vec![], // Gas Objects
        builder.finish(),
        10_000, // Gas Budget
        1,      // Gas Price
    );

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature = context.config.keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?;

    signatures.push(signature);

    let response = context
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    // We know `init_warehouse` move function will create 1 object.
    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

    assert!(effects.status.is_ok());

    let warehouse_id = effects.created.first().unwrap().reference.object_id;

    Ok(warehouse_id)
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}

pub async fn handle_mint_nft(
    // sui: Arc<SuiClient>,
    keystore: Arc<Keystore>,
    // nft_data: NftData,
    package_id: Arc<String>,
    // warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    sender: SuiAddress,
    // mint_cap: Arc<String>,
) -> JoinHandle<Result<Vec<ObjectID>, RustSdkError>> {
    tokio::spawn(async move {
        mint_nft(
            // sui,
            keystore,
            // nft_data,
            package_id,
            // warehouse_id,
            module_name,
            gas_budget,
            sender,
            // mint_cap,
        )
        .await
    })
}

pub async fn mint_nft(
    // sui: Arc<SuiClient>,
    keystore: Arc<Keystore>,
    // nft_data: NftData,
    package_id: Arc<String>,
    // warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    // SuiAddress implements Copy
    sender: SuiAddress,
    // mint_cap: Arc<String>,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let context = get_context().await.unwrap();
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;
    // let warehouse_id = ObjectID::from_str(warehouse_id.as_str())
    //     .map_err(|err| err::object_id(err, warehouse_id.as_str()))?;
    // let mint_cap_id = ObjectID::from_str(mint_cap.as_str())
    //     .map_err(|err| err::object_id(err, mint_cap.as_str()))?;

    // let mut args = nft_data.to_map()?;

    // args.push(SuiJsonValue::from_object_id(mint_cap_id));
    // args.push(SuiJsonValue::from_object_id(warehouse_id));

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

        let print_gag = Gag::stderr().unwrap();

        // Get gas object
        let coins: Vec<ObjectRef> = context
            .gas_objects(sender)
            .await?
            .iter()
            // Ok to unwrap() since `get_gas_objects` guarantees gas
            .map(|(_val, object)| {
                // let gas = GasCoin::try_from(object).unwrap();
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

        drop(print_gag);

        // Sign transaction.
        let mut signatures: Vec<Signature> = vec![];
        signatures.push(keystore.sign_secure(
            &sender,
            &data,
            Intent::sui_transaction(),
        )?);

        // Execute the transaction.

        let response_ = context
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

    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

    // We know `mint_nft` move function will create 1 object.
    let nft_ids = effects.created;
    // .first().unwrap().reference.object_id;
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
    // sui: Arc<SuiClient>,
    keystore: Arc<Keystore>,
    // nft_data: NftData,
    package_id: Arc<String>,
    // warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    gas_coin: Arc<ObjectRef>,
    sender: SuiAddress,
    // index: usize,
    // mint_cap: Arc<String>,
) -> JoinHandle<Result<Vec<ObjectID>, RustSdkError>> {
    tokio::spawn(async move {
        mint_nft_with_gas_coin(
            // sui,
            keystore,
            // nft_data,
            package_id,
            // warehouse_id,
            module_name,
            gas_budget,
            gas_coin,
            sender,
            // index,
            // mint_cap,
        )
        .await
    })
}

pub async fn mint_nft_with_gas_coin(
    keystore: Arc<Keystore>,
    // nft_data: NftData,
    package_id: Arc<String>,
    // warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    gas_coin: Arc<ObjectRef>,
    // SuiAddress implements Copy
    sender: SuiAddress,
    // index: usize,
    // mint_cap: Arc<String>,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let context = get_context().await.unwrap();
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;

    let mut retry = 0;
    let response = loop {
        let mut builder = ProgrammableTransactionBuilder::new();

        // TODO: Change to 1_000
        for _ in 0..1_000 {
            builder.move_call(
                package_id,
                Identifier::new(module_name.as_str()).unwrap(),
                Identifier::new("airdrop_nft").unwrap(),
                vec![],
                vec![],
            )?;
        }

        // let print_gag = Gag::stderr().unwrap();

        let data = TransactionData::new_programmable(
            sender,
            vec![*gas_coin], // Gas Objects
            builder.finish(),
            *gas_budget, // Gas Budget
            1_000,       // Gas Price
        );

        // Sign transaction.
        let mut signatures: Vec<Signature> = vec![];
        signatures.push(keystore.sign_secure(
            &sender,
            &data,
            Intent::sui_transaction(),
        )?);

        // Execute the transaction.
        let response_ = context
            .execute_transaction_block(
                Transaction::from_data(
                    data,
                    Intent::sui_transaction(),
                    signatures,
                )
                .verify()?,
            )
            .await;

        // drop(print_gag);

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

    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

    // We know `mint_nft` move function will create 1 object.
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

pub async fn split(
    // coin_id: ObjectID,
    amount: Option<u64>,
    count: u64,
    gas_budget: u64,
) -> Result<(), RustSdkError> {
    let context = get_context().await.unwrap();
    let client = get_client().await.unwrap();
    // Maybe get these from the context
    let keystore = get_keystore().await.unwrap();
    let sender = get_active_address(&keystore).unwrap();

    let coin = select_coin(&client, sender).await?;

    if count <= 0 {
        return Err(RustSdkError::AnyhowError(anyhow!(
            "Coin split count must be greater than 0"
        )));
    }

    // Count is now 19
    let (count, split_amount) = if amount.is_some() {
        (count, amount.unwrap() / count)
    } else {
        (count - 1, coin.balance / count)
    };

    let split_amounts = vec![split_amount; count as usize];

    let data = client
        .transaction_builder()
        .split_coin(
            sender,
            coin.coin_object_id,
            split_amounts,
            None,
            gas_budget,
        )
        .await?;

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature = context.config.keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?;

    signatures.push(signature);

    let response = context
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

    assert!(effects.status.is_ok());

    Ok(())
}

pub async fn combine(gas_budget: u64) -> Result<(), RustSdkError> {
    let context = get_context().await.unwrap();
    let client = get_client().await.unwrap();
    // Maybe get these from the context
    let keystore = get_keystore().await.unwrap();
    let sender = get_active_address(&keystore).unwrap();

    let (main_coin, gas_coin, coins_to_merge) =
        get_main_coin_separated(&client, sender).await?;

    let pt =
        merge_coins(sender, &main_coin, &coins_to_merge, gas_budget).await?;

    let gas_coin_ref =
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

    let data = TransactionData::new_programmable(
        sender,
        vec![gas_coin_ref], // Gas Objects
        pt,
        gas_budget, // Gas Budget
        1_000,      // Gas Price
    );

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature = context.config.keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?;

    signatures.push(signature);

    let response = context
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

    assert!(effects.status.is_ok());

    Ok(())
}

pub async fn collect_royalties() {
    todo!()
}

pub async fn select_coin(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<Coin, RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();

    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

pub async fn get_main_coin_separated(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<(Coin, Coin, Vec<Coin>), RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let gas_id = ObjectID::from_str("0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940")
        .map_err(|err| err::object_id(err, "0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940"))?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_obj = coins.remove(gas_index);

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();
    // let coin_obj = coins.iter().find(|&c| c.balance == max_balance).unwrap();

    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok((coin_obj, gas_obj, coins))
}

pub async fn get_coin_separated(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<(Coin, Vec<Coin>), RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let gas_id = ObjectID::from_str("0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940")
        .map_err(|err| err::object_id(err, "0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940"))?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_obj = coins.remove(gas_index);

    Ok((gas_obj, coins))
}

pub async fn get_coins(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<Vec<Coin>, RustSdkError> {
    let mut coins: Vec<Coin> = vec![];
    let mut cursor = None;

    loop {
        let coin_page = client
            .coin_read_api()
            .get_coins(signer, Some("0x2::sui::SUI".into()), cursor, None)
            .await?;

        let Page {
            mut data,
            next_cursor,
            has_next_page,
        } = coin_page;

        coins.append(&mut data);

        if has_next_page {
            cursor = next_cursor;
        } else {
            break;
        }
    }

    Ok(coins)
}

pub async fn merge_coins(
    _signer: SuiAddress,
    main_coin: &Coin,
    coins_to_merge: &Vec<Coin>,
    // gas: &Coin,
    _gas_budget: u64,
) -> anyhow::Result<ProgrammableTransaction> {
    let mut builder = ProgrammableTransactionBuilder::new();

    let coins_to_merge_ref: Vec<ObjectRef> = coins_to_merge
        .iter()
        .map(|coin| (coin.coin_object_id, coin.version, coin.digest))
        .collect();
    let primary_coin_ref: ObjectRef = (
        main_coin.coin_object_id,
        main_coin.version,
        main_coin.digest,
    );
    // let gas_coin_ref = (gas.coin_object_id, gas.version, gas.digest);

    let coins_to_merge_args: Vec<CallArg> = coins_to_merge_ref
        .iter()
        .map(|coin_ref| CallArg::Object(ObjectArg::ImmOrOwnedObject(*coin_ref)))
        .collect();

    let main_arg =
        CallArg::Object(ObjectArg::ImmOrOwnedObject(primary_coin_ref));

    // call_args.append(&mut coins_to_merge_args);

    let type_param = TypeTag::from_str(main_coin.coin_type.as_str()).unwrap();
    let type_args = vec![type_param];
    let _gas_price = 1_000;

    coins_to_merge_args
        .iter()
        .map(|coin| {
            builder.move_call(
                SUI_FRAMEWORK_OBJECT_ID,              // Package ID
                coin::PAY_MODULE_NAME.to_owned(),     // Module Name
                coin::PAY_JOIN_FUNC_NAME.to_owned(),  // Function Name
                type_args.clone(),                    // Type Arguments
                vec![main_arg.clone(), coin.clone()], // Call Arguments
            )
        })
        .collect::<Result<()>>()?;

    // let _res = builder.move_call(
    //     SUI_FRAMEWORK_OBJECT_ID,             // Package ID
    //     coin::PAY_MODULE_NAME.to_owned(),    // Module Name
    //     coin::PAY_JOIN_FUNC_NAME.to_owned(), // Function Name
    //     type_args,                           // Type Arguments
    //     [main_arg, ],                           // Call Arguments
    // );

    Ok(builder.finish())
}
