use crate::utils::{execute_tx, get_reference_gas_price, CloneClient};
use crate::{err::RustSdkError, utils::get_context};
use anyhow::{anyhow, Result};
use std::fmt::Write;
use std::ops::Deref;
use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};
use sui_json_rpc_types::{Coin, Page, SuiTransactionBlockEffects};
use sui_sdk::{
    types::base_types::{ObjectID, SuiAddress},
    wallet_context::WalletContext,
    SuiClient,
};
use sui_types::transaction::{
    CallArg, ObjectArg, ProgrammableTransaction, TransactionData,
};
use sui_types::SUI_FRAMEWORK_PACKAGE_ID;
use sui_types::{
    base_types::ObjectRef,
    coin,
    gas_coin::MIST_PER_SUI,
    // messages::{CallArg, ObjectArg, ProgrammableTransaction, TransactionData},
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    TypeTag,
};

pub struct CoinList(Vec<(ObjectID, u64, f64)>);

impl Display for CoinList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();

        writeln!(
            writer,
            " {0: ^66} | {1: ^15} | {2: ^13} |",
            "Object ID", "Coin Value (MIST)", "Coin Value (SUI)"
        )?;
        writeln!(
            writer,
            "------------------------------------------------------------------------------------------------------------"
        )?;
        for coin in &self.0 {
            writeln!(
                writer,
                " {0: ^66} | {1: ^17} | {2: ^16} |",
                coin.0, coin.1, coin.2
            )?;
        }

        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}

pub async fn split(
    coin_id: ObjectID,
    amount: Option<u64>,
    count: u64,
    gas_budget: u64,
    gas_id: Option<ObjectID>,
) -> Result<(), RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();

    let data =
        prepare_split(&wallet_ctx, coin_id, amount, count, gas_budget, gas_id)
            .await?;

    let response = execute_tx(&wallet_ctx, data).await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    Ok(())
}

pub async fn prepare_split(
    wallet_ctx: &WalletContext,
    coin_id: ObjectID,
    amount: Option<u64>,
    count: u64,
    gas_budget: u64,
    gas_id: Option<ObjectID>,
) -> Result<TransactionData, RustSdkError> {
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let coin = select_coin(&client, sender, coin_id).await?;

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

    Ok(client
        .transaction_builder()
        .split_coin(
            sender,
            coin.coin_object_id,
            split_amounts,
            gas_id,
            gas_budget,
        )
        .await?)
}

pub async fn combine(
    gas_budget: u64,
    gas_id: ObjectID,
) -> Result<(), RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let data = prepare_combine(&client, sender, gas_budget, gas_id).await?;
    let response = execute_tx(&wallet_ctx, data).await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();
    assert!(effects.status.is_ok());

    Ok(())
}

pub async fn prepare_combine<C>(
    client: C,
    sender: SuiAddress,
    gas_budget: u64,
    gas_id: ObjectID,
) -> Result<TransactionData, RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let gas_price = get_reference_gas_price(client.clone_client()).await?;

    let (main_coin, gas_coin, coins_to_merge) =
        separate_gas_and_max_coin(client, sender, gas_id).await?;

    let pt =
        merge_coins(sender, &main_coin, &coins_to_merge, gas_budget).await?;

    let gas_coin_ref =
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

    Ok(TransactionData::new_programmable(
        sender,
        vec![gas_coin_ref], // Gas Objects
        pt,
        gas_budget,
        gas_price,
    ))
}

pub async fn select_coin<C>(
    client: C,
    sender: SuiAddress,
    coin_id: ObjectID,
) -> Result<Coin, RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let mut coins = get_coins(client.clone_client(), sender).await?;

    let index = coins
        .iter()
        .position(|c| c.coin_object_id == coin_id)
        .unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

pub async fn select_biggest_coin<C>(
    client: C,
    sender: SuiAddress,
) -> Result<Coin, RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let mut coins = get_coins(client, sender).await?;

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();

    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

pub async fn separate_gas_and_max_coin<C>(
    client: C,
    sender: SuiAddress,
    gas_id: ObjectID,
) -> Result<(Coin, Coin, Vec<Coin>), RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let mut coins = get_coins(client, sender).await?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_coin = coins.remove(gas_index);

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();
    let max_index =
        coins.iter().position(|c| c.balance == max_balance).unwrap();
    let max_coin = coins.remove(max_index);

    Ok((max_coin, gas_coin, coins))
}

pub async fn separate_gas_coin<C>(
    client: C,
    sender: SuiAddress,
    gas_id: ObjectID,
) -> Result<(Coin, Vec<Coin>), RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let mut coins = get_coins(client, sender).await?;

    let gas_index = coins
        .iter()
        .position(|c| c.coin_object_id == gas_id)
        .unwrap();

    let gas_obj = coins.remove(gas_index);

    Ok((gas_obj, coins))
}

pub async fn list_coins<C>(client: C, sender: SuiAddress) -> Result<CoinList>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let coins = get_coins(client, sender).await?;

    let list = coins
        .iter()
        .map(|coin| {
            (
                coin.coin_object_id,
                coin.balance,
                coin.balance as f64 / MIST_PER_SUI as f64,
            )
        })
        .collect();

    Ok(CoinList(list))
}

pub async fn get_coins(
    client: impl Deref<Target = SuiClient>,
    sender: SuiAddress,
) -> Result<Vec<Coin>, RustSdkError> {
    let mut coins: Vec<Coin> = vec![];
    let mut cursor = None;

    loop {
        let coin_page = client
            .coin_read_api()
            .get_coins(sender, Some("0x2::sui::SUI".into()), cursor, None)
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
    client: impl Deref<Target = SuiClient>,
    sender: SuiAddress,
) -> Result<Coin, RustSdkError> {
    let mut coins = get_coins(client, sender).await?;

    let max_balance = coins.iter().map(|c| c.balance).max().unwrap();
    let index = coins.iter().position(|c| c.balance == max_balance).unwrap();
    let coin_obj = coins.remove(index);

    Ok(coin_obj)
}

// TODO: Fix
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

    coins_to_merge_args.iter().try_for_each(|coin| {
        builder.move_call(
            SUI_FRAMEWORK_PACKAGE_ID,             // Package ID
            coin::PAY_MODULE_NAME.to_owned(),     // Module Name
            coin::PAY_JOIN_FUNC_NAME.to_owned(),  // Function Name
            type_args.clone(),                    // Type Arguments
            vec![main_arg.clone(), coin.clone()], // Call Arguments
        )
    })?;

    Ok(builder.finish())
}
