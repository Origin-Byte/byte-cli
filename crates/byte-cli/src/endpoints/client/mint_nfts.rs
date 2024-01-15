use crate::{
    endpoints::client::check_network_match,
    io::{LocalRead, LocalWrite},
    models::effects::{MintEffects, MintError, Minted},
};
use anyhow::{anyhow, Result};
use chrono::Local;
use console::style;
use gutenberg_types::Schema;
use indicatif::{ProgressBar, ProgressStyle};
use package_manager::Network;
use rust_sdk::mint::MintEffect;
use rust_sdk::{
    metadata::{Metadata, StorableMetadata},
    models::project::Project,
};
use rust_sdk::{
    mint,
    utils::{get_active_address, get_context},
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use terminal_link::Link;

/// Asynchronously mints NFTs to a warehouse.
///
/// # Arguments
/// * `schema` - Reference to the Schema struct representing the NFT schema.
/// * `gas_budget` - The gas budget for the minting operation.
/// * `warehouse_id` - Optional String representing the warehouse ID.
/// * `mint_cap_id` - Optional String representing the mint capability ID.
/// * `metadata_path` - PathBuf to the metadata JSON file.
/// * `state` - The current Project state.
/// * `amount` - The number of NFTs to mint.
/// * `batches` - The number of batches to divide the minting process into.
/// * `network` - Reference to the Network struct representing the blockchain
///   network.
///
/// # Returns
/// Result containing the updated Project state or an error.
///
/// # Functionality
/// - Initializes the process and checks network compatibility.
/// - Collects NFT metadata and filters already minted NFTs.
/// - Splits the minting process into batches and executes minting.
/// - Updates the state with the minting effects and writes logs.
/// - Prints a summary of the minting process and outputs links for exploration.
pub async fn mint_nfts(
    schema: &Schema,
    gas_budget: usize,
    warehouse_id: Option<String>,
    mint_cap_id: Option<String>,
    metadata_path: PathBuf,
    state: Project,
    amount: u64,
    batches: u64,
    network: &Network,
) -> Result<Project> {
    let contract_id = Arc::new(state.package_id.as_ref().unwrap().to_string());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = Arc::new(get_context().await.unwrap());

    check_network_match(&wallet_ctx, network)?;

    let active_address =
        get_active_address(&wallet_ctx.config.keystore).unwrap();

    let module_name = Arc::new(schema.package_name());

    println!("{} Collecting NFT metadata", style("WIP").cyan().bold());
    let nft_data = StorableMetadata::read_json(&metadata_path)?;
    println!("{} Collecting NFT metadata", style("DONE").green().bold());

    let warehouse = Arc::new(match warehouse_id {
        Some(warehouse) => warehouse,
        None => state
            .collection_objects
            .as_ref()
            .unwrap()
            .warehouses
            .first()
            .unwrap()
            .to_string(),
    });

    let mint_cap = Arc::new(match mint_cap_id {
        Some(mint_cap) => mint_cap,
        None => state
            .admin_objects
            .as_ref()
            .unwrap()
            .mint_caps
            .first()
            .unwrap()
            .id
            .to_string(),
    });

    let mint_path = metadata_path.parent().unwrap().join("minted.json");
    let mut minted: Minted = if Path::new(&mint_path).exists() {
        Minted::read_json(&mint_path)?
    } else {
        Minted::default()
    };

    // Filter already minted
    let to_mint: BTreeMap<u32, Metadata> = nft_data
        .0
        .into_iter()
        .filter(|(v, _k)| !minted.0.contains(v))
        .take(amount as usize)
        .collect();

    let (mut keys, mut meta): (Vec<u32>, Vec<Metadata>) =
        to_mint.into_iter().unzip();

    let jobs_no = keys.len() as u64;
    let mut failed_jobs = 0;

    // TODO: Current limitation, handle cases where division has remainder
    // the last batch should adapt the size to fit the `jobs_no`
    let batch_size = jobs_no / batches;

    let progress_bar = ProgressBar::new(jobs_no);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .progress_chars("#>-"),
    );

    let mut i = 0;
    let mut batches = vec![];
    let mut stack = vec![];
    let last_key = *keys.last().unwrap();

    println!("{} Preparing Metadata", style("WIP").cyan().bold());
    for key in keys.drain(..) {
        stack.push((key, meta.pop().unwrap()));
        i += 1;

        if i == batch_size || key == last_key {
            // Flush the batch
            let mut new_batch = vec![];

            stack.drain(..).for_each(|(k, v)| new_batch.push((k, v)));

            // Inverse order. Batches are stored as
            // batch 1: [100..1]
            // batch 2: [200..101]
            // ...
            new_batch.reverse();
            batches.push(new_batch);

            // Reset circuit
            i = 0;
        }
    }

    if !meta.is_empty() {
        return Err(anyhow!("An error has occurred while processing metadata"));
    }
    if !keys.is_empty() {
        return Err(anyhow!("An error has occurred while processing metadata"));
    }
    println!("{} Preparing Metadata", style("DONE").green().bold());

    println!("{} Minting NFTs on-chain", style("WIP").cyan().bold());
    let mut effects = MintEffects::default();

    for batch in batches.drain(..) {
        let from_nft = batch.last().unwrap().0;
        let to_nft = batch.first().unwrap().0;

        let effect = mint::mint_nfts_to_warehouse(
            batch,
            wallet_ctx.clone(),
            contract_id.clone(),
            module_name.clone(),
            gas_budget as u64,
            None, // Gas coin
            active_address,
            warehouse.clone(),
            mint_cap.clone(),
        )
        .await?;

        progress_bar.inc(batch_size);

        match effect {
            MintEffect::Success(mut nft_ids) => {
                effects.minted_nfts.append(&mut nft_ids);

                minted.0.append(&mut (from_nft..=to_nft).collect())
            }
            MintEffect::Error(error) => {
                failed_jobs += batch_size;

                effects
                    .error_logs
                    .push(MintError::new(from_nft, to_nft, error));
            }
        }
    }

    // Finish the progress bar
    progress_bar.finish_at_current_pos();

    println!("{} Minting NFTs on-chain", style("DONE").green().bold());

    let network = wallet_ctx.config.active_env.as_ref().unwrap();

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network={}",
        warehouse, network
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your NFTs on the {}",
        style(link).blue().bold().underlined(),
    );

    let mut log_path = metadata_path.parent().unwrap().to_path_buf();
    let now = Local::now().format("%Y%m%d%H%M%S").to_string();
    log_path.push(format!("logs/mint-{}.json", now));
    effects.write_json(log_path.as_path())?;
    minted.write_json(mint_path.as_path())?;

    let uploaded = jobs_no - failed_jobs;

    println!("Upload Summary");
    println!("--------------------------");
    println!(
        "{} {} out of {}",
        style("UPLOADED ").green().bold(),
        uploaded,
        jobs_no
    );

    if failed_jobs > 0 {
        println!(
            "{} {} out of {}",
            style("FAILED ").red().bold(),
            failed_jobs,
            jobs_no
        );
    }

    Ok(state)
}
