use crate::{
    coin::select_biggest_coin,
    consts::RECIPIENT_ADDRESS,
    err::RustSdkError,
    utils::{
        execute_tx, execute_tx_with_client, get_client_mainnet, get_context,
    },
};
use anyhow::Result;
use std::str::FromStr;
use sui_json_rpc_types::SuiTransactionBlockEffects;
use sui_sdk::{types::base_types::SuiAddress, SuiClient};
use sui_types::transaction::TransactionData;

/// Asynchronously pays a specified amount of SUI on mainnet
///
/// # Arguments
/// * `amount` - The amount of SUI to be paid.
/// * `gas_budget` - The budget for gas in the transaction.
///
/// # Returns
/// A result indicating success or an error (`RustSdkError`).
pub async fn pay_mainnet(
    amount: u64,
    gas_budget: u64,
    sender: SuiAddress,
) -> Result<(), RustSdkError> {
    let client = get_client_mainnet().await?;

    let data = prepare_pay(&client, sender, amount, gas_budget).await?;
    let response = execute_tx_with_client(&client, data, sender).await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();
    assert!(effects.status.is_ok());

    Ok(())
}

// Asynchronously pays a specified amount of SUI.
///
/// # Arguments
/// * `amount` - The amount of SUI to be paid.
/// * `gas_budget` - The budget for gas in the transaction.
///
/// # Returns
/// A result indicating success or an error (`RustSdkError`).
pub async fn pay(amount: u64, gas_budget: u64) -> Result<(), RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let data = prepare_pay(&client, sender, amount, gas_budget).await?;
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
    client: &SuiClient,
    sender: SuiAddress,
    amount: u64,
    gas_budget: u64,
) -> Result<TransactionData, RustSdkError> {
    let coin = select_biggest_coin(client, sender).await?;
    let ob_addr = SuiAddress::from_str(RECIPIENT_ADDRESS)?;

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
