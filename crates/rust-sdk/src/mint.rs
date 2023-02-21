use crate::err::{self, RustSdkError};
use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use sui_keys::keystore::{AccountKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    SuiClient,
};
use sui_types::intent::Intent;
use sui_types::messages::ExecuteTransactionRequestType;

use tokio::task::{AbortHandle, JoinHandle, JoinSet};

pub const NFT_PROTOCOL: &str = "0xa672b029392522d76849990dfcf72d9249d1d522";
pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct NftData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub attribute_keys: Option<Vec<String>>,
    pub attribute_values: Option<Vec<String>>,
}

impl NftData {
    pub fn to_map(&self) -> Vec<SuiJsonValue> {
        let mut map: Vec<SuiJsonValue> = Vec::new();

        for (_, value) in self.iter_fields().enumerate() {
            if let Some(value_) = value.downcast_ref::<Option<String>>() {
                let value =
                    SuiJsonValue::from_str(value_.as_ref().unwrap()).unwrap();

                map.push(value);
            }
        }
        map
    }
}

pub async fn create_warehouse(
    sui: &SuiClient,
    keystore: &Keystore,
) -> Result<String, RustSdkError> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(NFT_PROTOCOL)
        .map_err(|err| err::object_id(err, NFT_PROTOCOL))?;

    let call = sui
        .transaction_builder()
        .move_call(
            address,
            package_id,
            "warehouse",
            "init_warehouse",
            vec![],
            vec![],
            None,  // Gas object, Node can pick on itself
            10000, // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.
    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature)
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
) -> Result<ObjectID, RustSdkError> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;

    let warehouse_id = ObjectID::from_str(warehouse_id.as_str())
        .map_err(|err| err::object_id(err, warehouse_id.as_str()))?;

    let mut args = nft_data.to_map();

    args.push(SuiJsonValue::from_object_id(warehouse_id));

    let call = sui
        .transaction_builder()
        .move_call(
            address,
            package_id,
            module_name.as_str(),
            "mint_nft",
            vec![],
            args,
            None,        // Gas object, Node can pick on itself
            *gas_budget, // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.

    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature)
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
