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

/// Asynchronously creates a warehouse in a blockchain network.
///
/// # Arguments
/// * `schema` - Reference to the Schema struct representing the NFT schema.
/// * `gas_budget` - Optional usize representing the gas budget.
/// * `gas_coin` - Optional String representing the gas coin.
/// * `state` - Mutable reference to the Project struct.
/// * `network` - Reference to the Network struct representing the blockchain network.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
///
/// # Functionality
/// - Initializes the process by getting the wallet context and client.
/// - Checks if the network matches.
/// - Creates a MoveType representing the collection.
/// - Retrieves the gas coin and budget.
/// - Mints a warehouse object.
/// - Updates the state with the new warehouse object ID.
pub async fn create_warehouse(
    schema: &Schema,
    gas_budget: Option<usize>,
    gas_coin: Option<String>,
    state: &mut Project,
    network: &Network,
) -> Result<()> {
    // Gets the contract ID from the project state.
    let contract_id = state.package_id.as_ref().unwrap().to_string();
    println!("Initializing process on contract ID: {:?}", contract_id);

    // Retrieves wallet context and client.
    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    // Checks if the client's network matches with the specified network.
    check_network_match(&wallet_ctx, network)?;

    // Prints a work-in-progress message.
    println!("{} Creating warehouse", style("WIP").cyan().bold());

    // Constructs a MoveType for the collection.
    let collection_type = MoveType::new(
        contract_id.clone(),
        schema.package_name(),
        String::from(schema.nft().type_name()),
    );

    // Gets the launchpad package ID.
    let launchpad_pkg = ObjectID::from_str(get_launchpad_id(&network))?;

    // Retrieves the gas coin.
    let gas_coin = get_gas_coin(&client, sender, gas_coin).await?;
    let gas_coin_ref =
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

    // Calculates the gas budget.
    let gas_budget = get_gas_budget(gas_coin, gas_budget)?;

    // Mints the warehouse object and retrieves its ID.
    let warehouse_object_id = mint::create_warehouse(
        collection_type,
        launchpad_pkg,
        gas_coin_ref,
        gas_budget as u64,
    )
    .await
    .unwrap();

    // Prints a completion message.
    println!("{} Creating warehouse", style("DONE").green().bold());

    // Displays the warehouse object ID.
    println!("Warehouse object ID: {}", warehouse_object_id);

    // Creates a link to the Sui Explorer for the warehouse object.
    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network={}",
        warehouse_object_id.clone(),
        network
    );
    let link = Link::new("Sui Explorer", explorer_link.as_str());

    // Prints the explorer link.
    println!(
        "You can now find your warehouse object on the {}",
        style(link).blue().bold().underlined(),
    );

    // Updates the project state with the new warehouse object ID.
    let col_objs = state
        .collection_objects
        .get_or_insert(CollectionObjects::empty());
    col_objs.warehouses.push(warehouse_object_id);

    Ok(())
}
