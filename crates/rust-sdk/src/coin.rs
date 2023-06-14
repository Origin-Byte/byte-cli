use crate::{
    err::{self, RustSdkError},
    utils::get_context,
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

pub async fn split(
    amount: Option<u64>,
    count: u64,
    gas_budget: u64,
) -> Result<(), RustSdkError> {
    let context = get_context().await.unwrap();
    let client = context.get_client().await?;
    let keystore = &context.config.keystore;
    let sender = context.config.active_address.unwrap();

    let coin = select_coin(&client, sender).await?;

    if count == 0 {
        return Err(RustSdkError::AnyhowError(anyhow!(
            "Coin split count must be greater than 0"
        )));
    }

    // TODO: Improve this flow is confusing
    let (count, split_amount) = if amount.is_some() {
        (count, amount.unwrap() / count)
    } else {
        (count - 1, coin.balance / count)
    };

    let split_amounts = vec![split_amount; count as usize];

    let data = client
        .transaction_builder()
        .split_coin(
            sender,
            coin.coin_object_id,
            split_amounts,
            None,
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

pub async fn combine(gas_budget: u64) -> Result<(), RustSdkError> {
    let context = get_context().await.unwrap();
    let client = context.get_client().await?;
    let keystore = &context.config.keystore;
    let sender = context.config.active_address.unwrap();

    let (main_coin, gas_coin, coins_to_merge) =
        get_main_coin_separated(&client, sender).await?;

    let pt =
        merge_coins(sender, &main_coin, &coins_to_merge, gas_budget).await?;

    let gas_coin_ref =
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

    let data = TransactionData::new_programmable(
        sender,
        vec![gas_coin_ref], // Gas Objects
        pt,
        gas_budget, // Gas Budget
        1_000,      // Gas Price
    );

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

pub async fn select_coin(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<Coin, RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();

    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

pub async fn get_main_coin_separated(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<(Coin, Coin, Vec<Coin>), RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let gas_id = ObjectID::from_str("0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940")
        .map_err(|err| err::object_id(err, "0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940"))?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_obj = coins.remove(gas_index);

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();
    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok((coin_obj, gas_obj, coins))
}

pub async fn get_coin_separated(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<(Coin, Vec<Coin>), RustSdkError> {
    let mut coins = get_coins(client, signer).await?;

    let gas_id = ObjectID::from_str("0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940")
        .map_err(|err| err::object_id(err, "0x9c65a3fc6a4d66f64f883f64faf6335f9a55cbb145eaf0522cd92d6b2d0e6940"))?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_obj = coins.remove(gas_index);

    Ok((gas_obj, coins))
}

pub async fn get_coins(
    client: &SuiClient,
    signer: SuiAddress,
) -> Result<Vec<Coin>, RustSdkError> {
    let mut coins: Vec<Coin> = vec![];
    let mut cursor = None;

    loop {
        let coin_page = client
            .coin_read_api()
            .get_coins(signer, Some("0x2::sui::SUI".into()), cursor, None)
            .await?;

        let Page {
            mut data,
            next_cursor,
            has_next_page,
        } = coin_page;

        coins.append(&mut data);

        if has_next_page {
            cursor = next_cursor;
        } else {
            break;
        }
    }

    Ok(coins)
}

pub async fn get_max_coin(
    wallet_ctx: &WalletContext,
) -> Result<Coin, RustSdkError> {
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let mut coins = get_coins(&client, sender).await?;

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();
    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

pub async fn merge_coins(
    _signer: SuiAddress,
    main_coin: &Coin,
    coins_to_merge: &[Coin],
    _gas_budget: u64,
) -> anyhow::Result<ProgrammableTransaction> {
    let mut builder = ProgrammableTransactionBuilder::new();

    let coins_to_merge_ref: Vec<ObjectRef> = coins_to_merge
        .iter()
        .map(|coin| (coin.coin_object_id, coin.version, coin.digest))
        .collect();
    let primary_coin_ref: ObjectRef = (
        main_coin.coin_object_id,
        main_coin.version,
        main_coin.digest,
    );

    let coins_to_merge_args: Vec<CallArg> = coins_to_merge_ref
        .iter()
        .map(|coin_ref| CallArg::Object(ObjectArg::ImmOrOwnedObject(*coin_ref)))
        .collect();

    let main_arg =
        CallArg::Object(ObjectArg::ImmOrOwnedObject(primary_coin_ref));

    let type_param = TypeTag::from_str(main_coin.coin_type.as_str()).unwrap();
    let type_args = vec![type_param];
    let _gas_price = 1_000;

    coins_to_merge_args.iter().try_for_each(|coin| {
        builder.move_call(
            SUI_FRAMEWORK_OBJECT_ID,              // Package ID
            coin::PAY_MODULE_NAME.to_owned(),     // Module Name
            coin::PAY_JOIN_FUNC_NAME.to_owned(),  // Function Name
            type_args.clone(),                    // Type Arguments
            vec![main_arg.clone(), coin.clone()], // Call Arguments
        )
    })?;

    Ok(builder.finish())
}
