use crate::consts::get_launchpad_id;
use crate::endpoints::client::{
    check_network_match, get_gas_budget, get_gas_coin,
};
use anyhow::Result;
use console::style;
use gutenberg_types::Schema;
use package_manager::Network;
use rust_sdk::models::project::{CollectionObjects, Project};
use rust_sdk::utils::MoveType;
use rust_sdk::{mint, utils::get_context};
use std::str::FromStr;
use sui_sdk::types::base_types::ObjectID;
use terminal_link::Link;

pub async fn create_warehouse(
    schema: &Schema,
    gas_budget: Option<usize>,
    gas_coin: Option<String>,
    state: &mut Project,
    network: &Network,
) -> Result<()> {
    let contract_id = state.package_id.as_ref().unwrap().to_string();
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    check_network_match(&wallet_ctx, network)?;

    println!("{} Creating warehouse", style("WIP").cyan().bold());
    let collection_type = MoveType::new(
        contract_id.clone(),
        schema.package_name(),
        String::from(schema.nft().type_name()),
    );

    let launchpad_pkg = ObjectID::from_str(get_launchpad_id(&network))?;

    let gas_coin = get_gas_coin(&client, sender, gas_coin).await?;
    let gas_coin_ref =
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);
    let gas_budget = get_gas_budget(gas_coin, gas_budget)?;

    let warehouse_object_id = mint::create_warehouse(
        collection_type,
        launchpad_pkg,
        gas_coin_ref,
        gas_budget as u64,
    )
    .await
    .unwrap();

    println!("{} Creating warehouse", style("DONE").green().bold());

    println!("Warehouse object ID: {}", warehouse_object_id);

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network={}",
        warehouse_object_id.clone(),
        network
    );
    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your warehouse object on the {}",
        style(link).blue().bold().underlined(),
    );

    let col_objs = state
        .collection_objects
        .get_or_insert(CollectionObjects::empty());

    col_objs.warehouses.push(warehouse_object_id);

    Ok(())
}
