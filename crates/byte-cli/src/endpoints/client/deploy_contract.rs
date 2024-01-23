use anyhow::{anyhow, Result};
use console::style;
use package_manager::Network;
use rust_sdk::models::project::{
    AdminObjects, CollectionObjects, MintCap, Project,
};
use rust_sdk::utils::execute_tx;
use rust_sdk::{collection_state::ObjectType as OBObjectType, publish};
use rust_sdk::{consts::VOLCANO_EMOJI, utils::get_context};
use std::fs::File;
use std::path::Path;
use std::sync::mpsc::channel;
use sui_sdk::rpc_types::{
    SuiTransactionBlockEffects, SuiTransactionBlockResponse,
};
use terminal_link::Link;
use tokio::task::JoinSet;

use super::check_network_match;
use super::get_gas_budget;
use super::get_gas_coin;

/// Parses the state file to return a Project object.
///
/// # Arguments
/// * `config_file` - A reference to a Path representing the state file.
///
/// # Returns
/// Result containing the Project object or an error if parsing fails.
pub fn parse_state(config_file: &Path) -> Result<Project, anyhow::Error> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find state file "{}": {err}
Call `byte-cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Project>(file)
        .map_err(|err| anyhow!(r#"Failed to serialize project state: {err}."#))
}

/// Asynchronously publishes a contract.
///
/// # Arguments
/// * `state` - Mutable reference to the Project struct.
/// * `gas_coin` - Optional String representing the gas coin.
/// * `gas_budget` - Optional usize representing the gas budget.
/// * `network` - The blockchain network.
/// * `contract_dir` - A reference to a Path representing the contract
///   directory.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
pub async fn publish_contract(
    state: &mut Project,
    gas_coin: Option<String>,
    gas_budget: Option<usize>,
    network: Network,
    contract_dir: &Path,
) -> Result<()> {
    let wallet_ctx = rust_sdk::utils::get_context().await?;
    let sender = wallet_ctx.config.active_address.unwrap();
    let sui_client = wallet_ctx.get_client().await.unwrap();

    check_network_match(&wallet_ctx, &network)?;

    if let Some(pkg_id) = state.package_id {
        return Err(anyhow!(format!(
            "Collection has already been deploy: {}",
            pkg_id
        )));
    }

    // The project owner should be the publisher address
    state.project_owner = sender;

    let gas_coin = get_gas_coin(&sui_client, sender, gas_coin).await?;
    let gas_budget = get_gas_budget(gas_coin.clone(), gas_budget)?;

    let tx_data = publish::prepare_publish_contract(
        sender,
        contract_dir,
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest),
        gas_budget as u64,
    )
    .await?;

    let response: SuiTransactionBlockResponse =
        execute_tx(&wallet_ctx, tx_data).await?;

    process_effects(state, response).await?;

    Ok(())
}

/// Asynchronously processes the effects of a transaction block response.
///
/// # Arguments
/// * `state` - Mutable reference to the Project struct.
/// * `response` - SuiTransactionBlockResponse object.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
pub async fn process_effects(
    state: &mut Project,
    response: SuiTransactionBlockResponse,
) -> Result<(), anyhow::Error> {
    let context = get_context().await.unwrap();
    let client = context.get_client().await?;

    println!(
        "{} {}",
        VOLCANO_EMOJI,
        style("Contract has been successfuly deployed on-chain.")
            .green()
            .bold()
    );
    let mut set = JoinSet::new();

    // Creating a channel to send message with package ID
    let (sender, receiver) = channel();

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    if effects.status.is_err() {
        return Err(anyhow!("Transaction Failed: {:?}", effects));
    }

    let objects_created = effects.created;

    objects_created
        .iter()
        .map(|object| {
            // TODO: Remove this clone
            let object_ = object.clone();
            let client_ = client.clone();
            let sender_ = sender.clone();
            set.spawn(async move {
                publish::print_object(&client_, &object_, sender_).await;
            });
        })
        .for_each(drop);

    let mut i = 1;
    while let Some(res) = set.join_next().await {
        res.unwrap();
        i += 1;
    }

    println!("A total of {} object have been created.", i);

    let mut j = 0;

    // It's three as we are interest in the MintCap, Collection and Package
    // We need to make sure we agree on the number of objects that are recorded
    while j < 3 {
        let object_type = receiver.recv().unwrap();
        match object_type {
            OBObjectType::Package(object_id) => {
                state.package_id = Some(object_id);
                j += 1;
            }
            OBObjectType::MintCap(object_id) => {
                let admin_objs =
                    state.admin_objects.get_or_insert(AdminObjects::empty());
                admin_objs.mint_caps.push(MintCap::new(object_id));
                j += 1;
            }
            OBObjectType::Collection(object_id) => {
                let col_objs = state
                    .collection_objects
                    .get_or_insert(CollectionObjects::empty());

                col_objs.collection = Some(object_id);

                j += 1;
            }
            _ => {}
        }
    }

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network=testnet",
        state.package_id.as_ref().unwrap()
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(())
}
