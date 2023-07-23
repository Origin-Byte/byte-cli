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

// TODO: Add back feature
// pub async fn parallel_mint_nfts(
//     project_name: String,
//     gas_budget: usize,
//     state: Project,
//     main_gas_id: ObjectID,
//     minor_gas_id: ObjectID,
// ) -> Result<Project> {
//     let contract_id = Arc::new(state.package_id.as_ref().unwrap().to_string());
//     println!("Initiliazing process on contract ID: {:?}", contract_id);

//     let wallet_ctx = Arc::new(get_context().await.unwrap());
//     let client = Arc::new(wallet_ctx.get_client().await?);
//     let active_address = wallet_ctx.config.active_address.unwrap();
//     let gas_budget_ref = Arc::new(gas_budget as u64);
//     let project_name = Arc::new(project_name);

//     let mut set = JoinSet::new();

//     println!(
//         "{} Minting 100,000 NFTs on-chain",
//         style("WIP").cyan().bold()
//     );

//     // TODO: Generalize
//     let split = 100;
//     let split_budget = 500000000_u64;
//     let combine_budget = 500000000_u64;

//     coin::split(main_gas_id, None, split, split_budget, Some(minor_gas_id))
//         .await?;

//     let (_, mut coins_to_merge) =
//         coin::separate_gas_coin(&client, active_address, minor_gas_id).await?;

//     assert!(coins_to_merge.len() == split as usize);

//     let mut j = 0;

//     for i in 0..split {
//         let gas_coin = coins_to_merge.pop().unwrap();
//         let gas_coin_ref =
//             (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

//         set.spawn(
//             mint::handle_parallel_mint_nft(
//                 wallet_ctx.clone(),
//                 contract_id.clone(),
//                 project_name.clone(),
//                 gas_budget_ref.clone(),
//                 Arc::new(gas_coin_ref),
//                 active_address,
//             )
//             .await,
//         );

//         j += 1;

//         if j == 10 {
//             let ten_millis = time::Duration::from_millis(1000);
//             thread::sleep(ten_millis);
//             j = 0;
//         } else {
//             let ten_millis = time::Duration::from_millis(100);
//             thread::sleep(ten_millis);
//         }

//         if i == 50 {
//             let ten_millis = time::Duration::from_millis(2000);
//             thread::sleep(ten_millis);
//         }
//     }

//     while let Some(res) = set.join_next().await {
//         res.unwrap().unwrap().unwrap();
//     }

//     let ten_millis = time::Duration::from_millis(1_000);
//     thread::sleep(ten_millis);

//     coin::combine(combine_budget, minor_gas_id).await?;

//     println!(
//         "{} Minting 100,000 NFTs on-chain",
//         style("DONE").green().bold()
//     );

//     Ok(state)
// }
