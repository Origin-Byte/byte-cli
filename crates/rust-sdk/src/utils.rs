use std::str::FromStr;

use anyhow::{anyhow, Result};
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_json_rpc_types::Coin;
use sui_keys::keystore::AccountKeystore;
use sui_keys::keystore::{FileBasedKeystore, Keystore};
use sui_sdk::wallet_context::WalletContext;
use sui_types::base_types::{ObjectID, ObjectRef, SuiAddress};

use crate::err::RustSdkError;

pub async fn get_context() -> Result<WalletContext, RustSdkError> {
    let config_path = sui_config_dir()?.join(SUI_CLIENT_CONFIG);

    let ctx = WalletContext::new(&config_path, None).await?;

    Ok(ctx)
}

pub async fn get_reference_gas_price(
    wallet_ctx: &WalletContext,
) -> Result<u64, RustSdkError> {
    let gas_price = wallet_ctx
        .get_client()
        .await?
        .read_api()
        .get_reference_gas_price()
        .await?;

    Ok(gas_price)
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

pub fn get_coin_ref(coin: &Coin) -> ObjectRef {
    (coin.coin_object_id, coin.version, coin.digest)
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

pub fn get_object_id(obj_id_str: &str) -> ObjectID {
    ObjectID::from_str(obj_id_str).expect("Could not parse object ID")
}
