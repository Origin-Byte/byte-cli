use crate::RustSdkError;
use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};
use std::{fs::File, str::FromStr};
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
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

pub const NFT_PROTOCOL: &str = "0xa672b029392522d76849990dfcf72d9249d1d522";
pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct NftData {
    pub name: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub attribute_keys: Option<Vec<String>>,
    pub attribute_values: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

impl NftData {
    pub fn to_map(&self) -> Vec<SuiJsonValue> {
        let mut map: Vec<SuiJsonValue> = Vec::new();

        for (i, value) in self.iter_fields().enumerate() {
            let field_name = self.name_at(i).unwrap();

            if let Some(value_) = value.downcast_ref::<String>() {
                let value = SuiJsonValue::from_str(&value_).unwrap();

                map.push(value);
            }

            print!("Hey the field name is: {} \n", field_name);
            print!("Hey the value is: {:?} \n \n", value);
        }
        map
    }
}

pub async fn create_warehouse(
    sui: &SuiClient,
    keystore: &Keystore,
) -> Result<String, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(NFT_PROTOCOL)?;

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

    println!("Created new warehouse, with the id : [{}]", warehouse_id);
    println!("Warehouse owner: {}", address);

    Ok(warehouse_id.to_string())
}

pub async fn get_client() -> Result<SuiClient, anyhow::Error> {
    let client =
        SuiClient::new("https://fullnode.devnet.sui.io:443", None, None)
            .await?;

    Ok(client)
}

pub async fn get_keystore() -> Result<Keystore, anyhow::Error> {
    // Load keystore from ~/.sui/sui_config/sui.keystore
    let keystore_path = match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("Cannot obtain home directory path"),
    };

    let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    Ok(keystore)
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}

pub async fn mint_nft(
    sui: &SuiClient,
    keystore: &Keystore,
    nft_data: &NftData,
    package_id: &str,
    warehouse_id: &str,
    _module_name: &str,
) -> Result<ObjectID, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(package_id)?;
    let warehouse_id = ObjectID::from_str(warehouse_id)?;

    let mut args = nft_data.to_map();

    args.push(SuiJsonValue::from_object_id(warehouse_id));

    println!("Package ID: {}", package_id);

    // let args =
    //     args.iter()
    //         .map(|(arg_type, arg)| match arg_type {
    //             SuiArgType::StringSlice => SuiJsonValue::from_str(arg)
    //                 .map_err(|e| RustSdkError::error(e)),
    //             SuiArgType::ObjectId => {
    //                 let object_id = ObjectID::from_str(arg)
    //                     .map_err(|e| RustSdkError::error(e));

    //                 // Safe to unwrap as we have validated above
    //                 // that it is an Object ID
    //                 Ok(SuiJsonValue::from_object_id(object_id.unwrap()))
    //             }
    //         })
    //         .collect::<Result<Vec<SuiJsonValue>, RustSdkError>>()?;

    let call = sui
        .transaction_builder()
        .move_call(
            address,
            package_id,
            "suimarines",
            "mint_nft",
            vec![],
            args,
            None,   // Gas object, Node can pick on itself
            100000, // Gas budget
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

    println!("{:?}", response);

    // We know `mint_nft` move function will create 1 object.
    let nft_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new NFT, with the id : [{}]", nft_id);

    Ok(nft_id)
}

// pub async fn collect_royalties() {}
