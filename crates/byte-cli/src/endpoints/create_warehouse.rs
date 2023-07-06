use crate::{
    io,
    models::project::{CollectionObjects, Project},
};
use anyhow::{anyhow, Result};
use console::style;
use gutenberg_types::Schema;
use package_manager::{toml::MoveToml, Network};
use rust_sdk::{coin, utils::MoveType};
use rust_sdk::{mint, utils::get_context};
use std::str::FromStr;
use sui_sdk::types::base_types::ObjectID;
use terminal_link::Link;

pub async fn create_warehouse(
    schema: &Schema,
    gas_budget: usize,
    move_toml: &MoveToml,
    state: &mut Project,
) -> Result<()> {
    let contract_id = state.package_id.as_ref().unwrap().to_string();
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let network_string = wallet_ctx.config.active_env.as_ref().unwrap();
    let net = Network::from_str(network_string.as_str())
        .map_err(|_| anyhow!("Invalid network string"))?;

    let registry = io::get_program_registry(&net)?;

    println!("{} Creating warehouse", style("WIP").cyan().bold());
    let collection_type = MoveType::new(
        contract_id.clone(),
        schema.package_name(),
        String::from(schema.nft().type_name()),
    );

    let rev = move_toml.get_dependency("Launchpad").rev.clone();

    let launchpad_pkg = ObjectID::from_str(
        registry
            .get_object_id_from_rev(String::from("Launchpad"), rev)?
            .as_string(),
    )?;

    let gas_coin = rust_sdk::utils::get_coin_ref(
        &coin::get_max_coin(&client, sender).await?,
    );

    let warehouse_object_id = mint::create_warehouse(
        collection_type,
        launchpad_pkg,
        gas_coin,
        gas_budget as u64,
    )
    .await
    .unwrap();

    println!("{} Creating warehouse", style("DONE").green().bold());

    println!("Warehouse object ID: {}", warehouse_object_id);

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network={}",
        warehouse_object_id.clone(),
        net
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
