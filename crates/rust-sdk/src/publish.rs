use crate::err::RustSdkError;
use move_package::BuildConfig as MoveBuildConfig;
use std::{path::Path, str::FromStr};
use sui_framework::build_move_package;
use sui_framework_build::compiled_package::BuildConfig;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::{
    types::{base_types::SuiAddress, messages::Transaction},
    SuiClient,
};
use sui_types::intent::Intent;
use sui_types::messages::ExecuteTransactionRequestType;

pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

pub async fn publish_contract(
    sui: &SuiClient,
    keystore: &Keystore,
    _compiled_modules: Vec<Vec<u8>>,
) -> Result<String, RustSdkError> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;

    let compiled_package = build_move_package(
        Path::new("suimarines"),
        BuildConfig {
            config: MoveBuildConfig::default(),
            run_bytecode_verifier: true,
            print_diags_to_stderr: true,
        },
    )?;

    let compiled_modules = compiled_package.get_package_bytes();

    let call = sui
        .transaction_builder()
        .publish(
            address, // sender
            compiled_modules,
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

pub async fn get_client() -> Result<SuiClient, RustSdkError> {
    let client =
        SuiClient::new("https://fullnode.devnet.sui.io:443", None, None)
            .await?;

    Ok(client)
}

pub async fn get_keystore() -> Result<Keystore, RustSdkError> {
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
