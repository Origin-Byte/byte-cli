use anyhow::Result;
use console::style;
use gutenberg::Schema;
use package_manager::move_lib::PackageMap;
use package_manager::toml::MoveToml;
use package_manager::Network;
use rust_sdk::{coin, utils::MoveType};
use std::str::FromStr;
use std::sync::Arc;
use sui_sdk::types::base_types::ObjectID;
use terminal_link::Link;

use rust_sdk::{
    mint::{self},
    utils::{get_active_address, get_context},
};

use crate::models::project::{CollectionObjects, Project};

pub async fn create_warehouse(
    schema: &Schema,
    gas_budget: usize,
    move_toml: &MoveToml,
    // TODO: Get registry depending on the network
    registry: &PackageMap,
    state: &mut Project,
    network: &Network,
) -> Result<()> {
    let contract_id = state.package_id.as_ref().unwrap().to_string();
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = Arc::new(get_context().await.unwrap());
    let active_address =
        get_active_address(&wallet_ctx.config.keystore).unwrap();

    let gas_budget_ref = Arc::new(gas_budget as u64);

    println!("{} Creating warehouse", style("WIP").cyan().bold());
    let collection_type = MoveType::new(
        contract_id.clone(),
        schema.package_name(),
        String::from(schema.nft().type_name()),
    );

    let rev = move_toml.get_dependency("Launchpad").rev.clone();

    let launchpad_pkg = ObjectID::from_str(
        registry
            .get_object_id_from_ref(String::from("Launchpad"), rev)
            .as_string(),
    )?;

    let gas_coin =
        rust_sdk::utils::get_coin_ref(&coin::get_max_coin(&wallet_ctx).await?);

    let warehouse_object_id =
        mint::create_warehouse(collection_type, launchpad_pkg, gas_coin)
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
