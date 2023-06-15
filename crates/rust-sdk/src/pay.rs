use crate::{
    err::{self, RustSdkError},
    utils::{get_active_address, get_client, get_context, get_keystore},
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

pub async fn pay(amount: u64, gas_budget: u64) -> Result<(), RustSdkError> {
    let context = get_context().await.unwrap();
    let client = get_client().await.unwrap();
    // Maybe get these from the context
    let keystore = get_keystore().await.unwrap();
    let sender = get_active_address(&keystore).unwrap();

    let coin = select_coin(&client, sender).await?;

    let split_amounts = vec![split_amount; count as usize];

    let ob_addr = SuiAddress::from_str(
        "0xf9935ad63df83d84c5c071a65f241548ac7c452af2912218c6d2b3faefba5dc2",
    );

    let data = client
        .transaction_builder()
        .pay_sui(
            sender,                    // signer
            vec![coin.coin_object_id], // input_coins
            vec![ob_addr],             // recipients
            vec![amount],              // amounts
            gas_budget,
        )
        .await?;

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature = context.config.keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?;

    signatures.push(signature);

    let response = context
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    Ok(())
}
