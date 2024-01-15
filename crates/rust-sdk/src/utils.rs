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
    SuiTransactionBlockResponseOptions,
};
use sui_keys::keystore::AccountKeystore;
use sui_keys::keystore::{FileBasedKeystore, Keystore};
use sui_sdk::wallet_context::WalletContext;
use sui_sdk::{SuiClient, SuiClientBuilder};
use sui_types::base_types::{ObjectID, ObjectRef, SuiAddress};
use sui_types::crypto::Signature;
use sui_types::quorum_driver_types::ExecuteTransactionRequestType;
use sui_types::transaction::{Transaction, TransactionData};

use crate::err::RustSdkError;

pub async fn get_client_mainnet() -> Result<SuiClient, RustSdkError> {
    let client_builder = SuiClientBuilder::default();
    let client = client_builder
        .build("https://fullnode.mainnet.sui.io:443")
        .await?;

    Ok(client)
}

/// Retrieves a Sui client for network interactions.
///
/// # Arguments
/// * `uri` - The URI of the Sui fullnode to connect to.
///
/// # Returns
/// A result containing the Sui client or a `RustSdkError`.
pub async fn get_client(uri: &str) -> Result<SuiClient, RustSdkError> {
    // e.g. uri: https://fullnode.devnet.sui.io:443
    let client_builder = SuiClientBuilder::default();
    let client = client_builder.build(uri).await?;

    Ok(client)
}

/// Retrieves the wallet context from the local configuration.
///
/// # Returns
/// A result containing the wallet context (`WalletContext`) or a
/// `RustSdkError`.
pub async fn get_context() -> Result<WalletContext, RustSdkError> {
    let config_path = sui_config_dir()?.join(SUI_CLIENT_CONFIG);

    let ctx = WalletContext::new(&config_path, None, None).await?;

    Ok(ctx)
}

/// Retrieves the reference gas price from the network.
///
/// # Arguments
/// * `client` - An instance of `SuiClient` for network interaction.
///
/// # Returns
/// A result containing the reference gas price or a `RustSdkError`.
pub async fn get_reference_gas_price(
    client: impl Deref<Target = SuiClient>,
) -> Result<u64, RustSdkError> {
    let gas_price = client.read_api().get_reference_gas_price().await?;

    Ok(gas_price)
}

/// Get all the gas objects (and conveniently, gas amounts) for the address
/// Copied and modified from https://github.com/MystenLabs/sui/blob/main/crates/sui-sdk/src/wallet_context.rs#L119
///
/// Retrieves all gas objects and their values for a given address.
///
/// # Type Parameters
/// * `C` - The client type, bounded by `Deref` targeting `SuiClient` and
///   `CloneClient`.
///
/// # Arguments
/// * `client` - The client used for network interactions.
/// * `address` - The Sui address for which to retrieve gas objects.
///
/// # Returns
/// A result containing a vector of tuples with gas values and corresponding
/// object data.
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

/// Retrieves the keystore from the local configuration.
///
/// # Returns
/// A result containing the keystore or a `RustSdkError`.
pub fn get_keystore() -> Result<Keystore, RustSdkError> {
    // Load keystore from ~/.sui/sui_config/sui.keystore
    let keystore_path = match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("Cannot obtain home directory path"),
    };

    let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    Ok(keystore)
}

/// Retrieves the active address from the keystore.
///
/// # Arguments
/// * `keystore` - A reference to the keystore.
///
/// # Returns
/// A result containing the active Sui address or an error.
pub fn get_active_address(keystore: &Keystore) -> Result<SuiAddress> {
    keystore.addresses().last().cloned().ok_or_else(|| {
        anyhow!("Could not retrieve active address from keystore")
    })
}

/// Extracts the object reference from a `Coin` object.
///
/// # Arguments
/// * `coin` - A reference to the `Coin` object.
///
/// # Returns
/// The object reference (`ObjectRef`) for the coin.
pub fn get_coin_ref(coin: &Coin) -> ObjectRef {
    (coin.coin_object_id, coin.version, coin.digest)
}

/// Represents a Move type in the Sui ecosystem.
///
/// # Fields
/// * `contract_address` - The address of the contract.
/// * `module` - The module name within the contract.
/// * `type_name` - The type name within the module.
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

/// Converts a string to an `ObjectID`.
///
/// # Arguments
/// * `obj_id_str` - The string representation of the object ID.
///
/// # Returns
/// The `ObjectID` parsed from the string.
pub fn get_object_id(obj_id_str: &str) -> ObjectID {
    ObjectID::from_str(obj_id_str).expect("Could not parse object ID")
}

/// Executes a transaction using the provided wallet context and transaction
/// data.
///
/// This function signs and executes a transaction based on the given
/// transaction data. It handles both the signing process using the wallet's
/// keystore and the actual sending of the transaction to the network.
///
/// # Type Parameters
/// * `wallet_ctx` - A flexible parameter that can accept both
///   `Arc<WalletContext>` and a reference to `WalletContext`.
///
/// # Arguments
/// * `tx_data` - The transaction data to be executed, encapsulating details
///   like the transaction's payload and gas information.
///
/// # Returns
/// A result containing the transaction block response
/// (`SuiTransactionBlockResponse`) upon success, or a `RustSdkError` in case of
/// failure.
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
        style("WIP").cyan().bold()
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

pub async fn execute_tx_with_client(
    // This way it works with both Arc<SuiClient> and &SuiClient
    client: impl Deref<Target = SuiClient>,
    tx_data: TransactionData,
    sender: SuiAddress,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let keystore = get_keystore()?;

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature =
        keystore.sign_secure(&sender, &tx_data, Intent::sui_transaction())?;

    signatures.push(signature);

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIP").cyan().bold()
    );

    let response = client
        .quorum_driver_api()
        .execute_transaction_block(
            Transaction::from_data(
                tx_data,
                Intent::sui_transaction(),
                signatures,
            ),
            SuiTransactionBlockResponseOptions::new(),
            Some(ExecuteTransactionRequestType::WaitForEffectsCert),
        )
        .await?;

    println!(
        "{} Sending and executing transaction.",
        style("Done").cyan().bold()
    );

    Ok(response)
}

/// A trait to enable cloning of Sui client instances.
///
/// This trait provides a method to clone instances of Sui clients. It's useful
/// in contexts where a client instance needs to be reused or passed across
/// different parts of an application, especially when dealing with `Arc` for
/// shared ownership or references to the client.
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
