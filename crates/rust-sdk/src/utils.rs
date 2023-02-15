use sui_keys::keystore::{FileBasedKeystore, Keystore};
use sui_sdk::{SuiClient, SuiClientBuilder};

use crate::err::RustSdkError;

pub async fn get_client() -> Result<SuiClient, RustSdkError> {
    let client_builder = SuiClientBuilder::default();
    let client = client_builder
        .build("https://fullnode.devnet.sui.io:443")
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
