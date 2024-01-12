use crate::{
    err::{self, RustSdkError},
    utils::{get_active_address, get_context, get_keystore},
};
use anyhow::{anyhow, Result};
use shared_crypto::intent::Intent;
use std::str::FromStr;
use sui_json_rpc_types::{Coin, Page, SuiTransactionBlockEffects};
use sui_keys::keystore::AccountKeystore;
use sui_sdk::{
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    wallet_context::WalletContext,
    SuiClient,
};
use sui_types::{
    base_types::ObjectRef,
    coin,
    messages::{CallArg, ObjectArg, ProgrammableTransaction},
    TypeTag, SUI_FRAMEWORK_OBJECT_ID,
};
use sui_types::{
    crypto::Signature, messages::TransactionData,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};

/// Asynchronously pays a specified amount of SUI.
///
/// # Arguments
/// * `amount` - The amount of SUI to be paid.
/// * `gas_budget` - The budget for gas in the transaction.
///
/// # Returns
/// A result indicating success or an error (`RustSdkError`).
pub async fn pay(amount: u64, gas_budget: u64) -> Result<(), RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();

    let data = prepare_pay(&wallet_ctx, amount, gas_budget).await?;
    let response = execute_tx(&wallet_ctx, data).await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();
    assert!(effects.status.is_ok());

    Ok(())
}

/// Asynchronously prepares the data required for a payment transaction.
///
/// # Arguments
/// * `wallet_ctx` - A reference to the wallet context.
/// * `amount` - The amount of SUI to be paid.
/// * `gas_budget` - The budget for gas in the transaction.
///
/// # Returns
/// A result containing `TransactionData` on success or an error
/// (`RustSdkError`).
pub async fn prepare_pay(
    wallet_ctx: &WalletContext,
    amount: u64,
    gas_budget: u64,
) -> Result<TransactionData, RustSdkError> {
    let client = context.get_client().await?;
    let keystore = &context.config.keystore;
    let sender = context.config.active_address.unwrap();

    let coin = select_coin(&client, sender).await?;

    let split_amounts = vec![split_amount; count as usize];

    let ob_addr = SuiAddress::from_str(RECIPIENT_ADDRESS);

    Ok(client
        .transaction_builder()
        .pay_sui(
            sender,                    // signer
            vec![coin.coin_object_id], // input_coins
            vec![ob_addr],             // recipients
            vec![amount],              // amounts
            gas_budget,
        )
        .await?)
}
