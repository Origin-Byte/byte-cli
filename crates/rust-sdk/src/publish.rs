use crate::err::RustSdkError;
// use crate::sui::client_commands::WalletContext;
use move_package::BuildConfig as MoveBuildConfig;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use sui_config::{
    sui_config_dir, Config, PersistedConfig, FULL_NODE_DB_PATH,
    SUI_CLIENT_CONFIG, SUI_FULLNODE_CONFIG, SUI_NETWORK_CONFIG,
};
use sui_framework::build_move_package;
use sui_framework_build::compiled_package::BuildConfig;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::{
    types::{base_types::SuiAddress, messages::Transaction},
    SuiClient,
};
use sui_source_validation::{BytecodeSourceVerifier, SourceMode};
use sui_types::intent::Intent;
use sui_types::messages::ExecuteTransactionRequestType;

pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

pub async fn publish_contract(
    sui: &SuiClient,
    keystore: &Keystore,
) -> Result<String, RustSdkError> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let verify_dependencies = false;
    let package_path = Path::new("suimarines");

    let config_path = sui_config_dir()?.join(SUI_CLIENT_CONFIG);
    // let mut context = WalletContext::new(&config_path, None).await?;

    // let sender = context.try_get_object_owner(&gas).await?;
    // let sender = sender.unwrap_or(context.active_address()?);

    let compiled_modules_base_64 = BuildConfig::default()
        .build(Path::new("suimarines").to_path_buf())?
        .get_package_base64(false);

    let compiled_modules = compiled_modules_base_64
        .into_iter()
        .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<_>, _>>()?;

    println!("Prepare Publish transaction...");
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
    println!("Signing Publish transaction...");
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.
    println!("Preppin tx data...");
    let transaction_data =
        Transaction::from_data(call, Intent::default(), signature).verify()?;
    println!("Awaiting response...");

    let response = sui
        .quorum_driver()
        .execute_transaction(
            transaction_data,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await
        .unwrap();

    println!("Not sure...");
    assert!(response.confirmed_local_execution);

    println!("Querying package ID");
    let package_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new package, with the id : {}", package_id);

    Ok(package_id.to_string())
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

// async fn publish(
//     &self,
//     sender: SuiAddress,
//     compiled_modules: Vec<Base64>,
//     gas: Option<ObjectID>,
//     gas_budget: u64,
// ) -> RpcResult<TransactionBytes> {
//     let compiled_modules = compiled_modules
//         .into_iter()
//         .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
//         .collect::<Result<Vec<_>, _>>()?;
//     let data = self
//         .builder
//         .publish(sender, compiled_modules, gas, gas_budget)
//         .await?;
//     Ok(TransactionBytes::from_data(data)?)
// }
