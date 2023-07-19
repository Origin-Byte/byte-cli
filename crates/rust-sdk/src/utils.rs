use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;
use sui_types::gas_coin::GasCoin;

use anyhow::{anyhow, Result};
use console::style;
use shared_crypto::intent::Intent;
use sui_config::{sui_config_dir, SUI_CLIENT_CONFIG};
use sui_json_rpc_types::{
    Coin, SuiObjectData, SuiObjectDataFilter, SuiObjectDataOptions,
    SuiObjectResponse, SuiObjectResponseQuery, SuiTransactionBlockResponse,
};
use sui_keys::keystore::AccountKeystore;
use sui_keys::keystore::{FileBasedKeystore, Keystore};
use sui_sdk::wallet_context::WalletContext;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::{ObjectID, ObjectRef, SuiAddress};
use sui_types::crypto::Signature;
use sui_types::transaction::{Transaction, TransactionData};

use crate::err::RustSdkError;

pub async fn get_client(uri: &str) -> Result<SuiClient, RustSdkError> {
    // e.g. uri: https://fullnode.devnet.sui.io:443
    let client_builder = SuiClientBuilder::default();
    let client = client_builder.build(uri).await?;

    Ok(client)
}

pub async fn get_context() -> Result<WalletContext, RustSdkError> {
    let config_path = sui_config_dir()?.join(SUI_CLIENT_CONFIG);

    let ctx = WalletContext::new(&config_path, None, None).await?;

    Ok(ctx)
}

pub async fn get_reference_gas_price(
    client: impl Deref<Target = SuiClient>,
) -> Result<u64, RustSdkError> {
    let gas_price = client.read_api().get_reference_gas_price().await?;

    Ok(gas_price)
}

/// Get all the gas objects (and conveniently, gas amounts) for the address
/// Copied and modified from https://github.com/MystenLabs/sui/blob/main/crates/sui-sdk/src/wallet_context.rs#L119
pub async fn gas_objects<C>(
    client: C,
    address: SuiAddress,
) -> Result<Vec<(u64, SuiObjectData)>>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let mut objects: Vec<SuiObjectResponse> = Vec::new();
    let mut cursor = None;
    loop {
        let response = client
            .read_api()
            .get_owned_objects(
                address,
                Some(SuiObjectResponseQuery::new(
                    Some(SuiObjectDataFilter::StructType(GasCoin::type_())),
                    Some(SuiObjectDataOptions::full_content()),
                )),
                cursor,
                None,
            )
            .await?;

        objects.extend(response.data);

        if response.has_next_page {
            cursor = response.next_cursor;
        } else {
            break;
        }
    }

    let mut values_objects = Vec::new();

    for object in objects {
        let o = object.data;
        if let Some(o) = o {
            let gas_coin = GasCoin::try_from(&o)?;
            values_objects.push((gas_coin.value(), o.clone()));
        }
    }

    Ok(values_objects)
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

pub async fn execute_tx(
    // This way it works with both Arc<WalletContext> and &WalletContext
    wallet_ctx: impl Deref<Target = WalletContext>,
    tx_data: TransactionData,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let keystore = &wallet_ctx.config.keystore;
    let sender = wallet_ctx.config.active_address.unwrap();

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature =
        keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;

    signatures.push(signature);

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );

    let response = wallet_ctx
        .execute_transaction_may_fail(Transaction::from_data(
            tx_data,
            Intent::sui_transaction(),
            signatures,
        ))
        .await?;

    println!(
        "{} Sending and executing transaction.",
        style("Done").cyan().bold()
    );

    Ok(response)
}

pub trait CloneClient {
    fn clone_client(&self) -> Self;
}

impl CloneClient for &SuiClient {
    fn clone_client(&self) -> Self {
        *self
    }
}

impl CloneClient for Arc<SuiClient> {
    fn clone_client(&self) -> Self {
        Arc::clone(self)
    }
}
