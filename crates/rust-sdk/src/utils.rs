use anyhow::{anyhow, Result};
use sui_keys::keystore::AccountKeystore;
use sui_keys::keystore::{FileBasedKeystore, Keystore};
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::SuiAddress;

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

pub fn get_active_address(keystore: &Keystore) -> Result<SuiAddress> {
    keystore.addresses().last().cloned().ok_or_else(|| {
        anyhow!("Could not retrieve active address from keystore")
    })
}

pub struct MoveType {
    contract_address: String,
    module: String,
    type_name: String,
}

impl MoveType {
    pub fn new(
        contract_address: String,
        module: String,
        type_name: String,
    ) -> Self {
        MoveType {
            contract_address,
            module,
            type_name,
        }
    }

    pub fn write_type(&self) -> String {
        let mut code = self.contract_address.clone();
        code.push_str("::");
        code.push_str(self.module.as_str());
        code.push_str("::");
        code.push_str(self.type_name.as_str());

        code
    }
}
