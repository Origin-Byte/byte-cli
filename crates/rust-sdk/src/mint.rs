use crate::{
    consts::NFT_PROTOCOL,
    err::{self, RustSdkError},
    utils::MoveType,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::{collections::HashMap, str::FromStr};
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    SuiClient,
};
use sui_types::{crypto::Signature, messages::ExecuteTransactionRequestType};
use sui_types::{intent::Intent, parse_sui_type_tag};
use tokio::task::JoinHandle;

#[derive(Debug, Deserialize, Serialize)]
pub struct NftData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub attributes: Option<HashMap<String, String>>,
}

impl NftData {
    pub fn to_map(&self) -> Result<Vec<SuiJsonValue>> {
        let mut params: Vec<SuiJsonValue> = Vec::new();

        if let Some(value) = &self.name {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.description {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.url {
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
    sui: &SuiClient,
    keystore: &Keystore,
    // SuiAddress implements Copy
    sender: SuiAddress,
    collection_type: MoveType,
) -> Result<String, RustSdkError> {
    let package_id = ObjectID::from_str(NFT_PROTOCOL)
        .map_err(|err| err::object_id(err, NFT_PROTOCOL))?;

    let collection_type_ = collection_type.write_type();

    let call = sui
        .transaction_builder()
        .move_call(
            sender,
            package_id,
            "warehouse",
            "init_warehouse",
            vec![parse_sui_type_tag(collection_type_.as_str())?.into()], // type args
            vec![], // call args
            None,   // Gas object, Node can pick on itself
            10000,  // Gas budget
        )
        .await?;

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];
    signatures.push(keystore.sign_secure(&sender, &call, Intent::default())?);

    // Execute the transaction.
    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signatures)
                .verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

    // We know `init_warehouse` move function will create 1 object.
    let warehouse_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new warehouse, with the id : {}", warehouse_id);

    Ok(warehouse_id.to_string())
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}

pub async fn handle_mint_nft(
    sui: Arc<SuiClient>,
    keystore: Arc<Keystore>,
    nft_data: NftData,
    package_id: Arc<String>,
    warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    sender: SuiAddress,
    mint_cap: Arc<String>,
) -> JoinHandle<Result<ObjectID, RustSdkError>> {
    tokio::spawn(async move {
        mint_nft(
            sui,
            keystore,
            nft_data,
            package_id,
            warehouse_id,
            module_name,
            gas_budget,
            sender,
            mint_cap,
        )
        .await
    })
}

pub async fn mint_nft(
    sui: Arc<SuiClient>,
    keystore: Arc<Keystore>,
    nft_data: NftData,
    package_id: Arc<String>,
    warehouse_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: Arc<u64>,
    // SuiAddress implements Copy
    sender: SuiAddress,
    mint_cap: Arc<String>,
) -> Result<ObjectID, RustSdkError> {
    println!("Minting NFT function");
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;
    let warehouse_id = ObjectID::from_str(warehouse_id.as_str())
        .map_err(|err| err::object_id(err, warehouse_id.as_str()))?;
    let mint_cap_id = ObjectID::from_str(mint_cap.as_str())
        .map_err(|err| err::object_id(err, mint_cap.as_str()))?;
    println!("Warehouse ID: {:?}", warehouse_id);

    println!("NFT Data: {:?}", nft_data);
    let mut args = nft_data.to_map()?;

    args.push(SuiJsonValue::from_object_id(mint_cap_id));
    args.push(SuiJsonValue::from_object_id(warehouse_id));
    println!("Args are: {:?}", args);

    let call = sui
        .transaction_builder()
        .move_call(
            sender,
            package_id,
            module_name.as_str(),
            "mint_to_launchpad",
            vec![],
            args,
            None,        // Gas object, Node can pick on itself
            *gas_budget, // Gas budget
        )
        .await?;

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];
    signatures.push(keystore.sign_secure(&sender, &call, Intent::default())?);

    // Execute the transaction.

    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signatures)
                .verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

    // We know `mint_nft` move function will create 1 object.
    let nft_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    Ok(nft_id)
}

pub async fn collect_royalties() {
    todo!()
}
