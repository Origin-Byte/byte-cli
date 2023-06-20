use crate::io::LocalRead;
use anyhow::Result;
use console::style;
use gutenberg::Schema;
use rust_sdk::coin;
use rust_sdk::metadata::StorableMetadata;
use std::path::PathBuf;
use std::sync::Arc;
use std::{thread, time};
use sui_sdk::types::base_types::ObjectID;
use tokio::task::JoinSet;

use rust_sdk::{
    mint::{self},
    utils::{get_active_address, get_context},
};

use crate::models::project::Project;

pub async fn mint_nfts(
    schema: &Schema,
    gas_budget: usize,
    warehouse_id: Option<String>,
    mint_cap_id: Option<String>,
    metadata_path: PathBuf,
    state: Project,
) -> Result<Project> {
    let contract_id = state.package_id.as_ref().unwrap().to_string();
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = get_context().await.unwrap();
    let active_address =
        get_active_address(&wallet_ctx.config.keystore).unwrap();

    let module_name = schema.package_name();
    let gas_budget_ref = gas_budget as u64;

    println!("{} Collecting NFT metadata", style("WIP").cyan().bold());
    let mut nft_data = StorableMetadata::read_json(&metadata_path)?;
    println!("{} Collecting NFT metadata", style("DONE").green().bold());

    println!("{} Minting NFTs on-chain", style("WIP").cyan().bold());

    let warehouse = match warehouse_id {
        Some(warehouse) => warehouse,
        None => state
            .collection_objects
            .as_ref()
            .unwrap()
            .warehouses
            .first()
            .unwrap()
            .to_string(),
    };

    let mint_cap = match mint_cap_id {
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
    };

    let batch_size = 100;

    while !nft_data.0.is_empty() {
        let key_index = *nft_data.0.keys().nth(batch_size).unwrap_or(&u32::MAX);

        let batch = nft_data.0.split_off(&key_index);

        mint::mint_nfts_in_batch(
            batch,
            &wallet_ctx,
            contract_id.clone(),
            module_name.clone(),
            gas_budget as u64,
            active_address,
            warehouse.clone(),
            mint_cap.clone(),
        )
        .await?;
    }

    println!("{} Minting NFTs on-chain", style("DONE").green().bold());

    // println!("Warehouse object ID: {}", warehouse_id_ref.clone());

    // let explorer_link = format!(
    //     "https://explorer.sui.io/object/{}?network=devnet",
    //     warehouse_id_ref.clone()
    // );

    // let link = Link::new("Sui Explorer", explorer_link.as_str());

    // println!(
    //     "You can now find your NFTs on the {}",
    //     style(link).blue().bold().underlined(),
    // );

    Ok(state)
}

pub async fn parallel_mint_nfts(
    project_name: String,
    gas_budget: usize,
    state: Project,
    main_gas_id: ObjectID,
    minor_gas_id: ObjectID,
) -> Result<Project> {
    let contract_id = Arc::new(state.package_id.as_ref().unwrap().to_string());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = Arc::new(get_context().await.unwrap());
    let client = Arc::new(wallet_ctx.get_client().await?);
    let active_address = wallet_ctx.config.active_address.unwrap();
    let gas_budget_ref = Arc::new(gas_budget as u64);
    let project_name = Arc::new(project_name);

    let mut set = JoinSet::new();

    println!(
        "{} Minting 100,000 NFTs on-chain",
        style("WIP").cyan().bold()
    );

    // TODO: Generalize
    let split = 100;
    let split_budget = 500000000_u64;
    let combine_budget = 500000000_u64;

    coin::split(main_gas_id, None, split, split_budget, Some(minor_gas_id))
        .await?;

    let (_, mut coins_to_merge) =
        coin::separate_gas_coin(&client, active_address, minor_gas_id).await?;

    assert!(coins_to_merge.len() == split as usize);

    let mut j = 0;

    for i in 0..split {
        let gas_coin = coins_to_merge.pop().unwrap();
        let gas_coin_ref =
            (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

        set.spawn(
            mint::handle_parallel_mint_nft(
                wallet_ctx.clone(),
                contract_id.clone(),
                project_name.clone(),
                gas_budget_ref.clone(),
                Arc::new(gas_coin_ref),
                active_address,
            )
            .await,
        );

        j += 1;

        if j == 10 {
            let ten_millis = time::Duration::from_millis(1000);
            thread::sleep(ten_millis);
            j = 0;
        } else {
            let ten_millis = time::Duration::from_millis(100);
            thread::sleep(ten_millis);
        }

        if i == 50 {
            let ten_millis = time::Duration::from_millis(2000);
            thread::sleep(ten_millis);
        }
    }

    while let Some(res) = set.join_next().await {
        res.unwrap().unwrap().unwrap();
    }

    let ten_millis = time::Duration::from_millis(1_000);
    thread::sleep(ten_millis);

    coin::combine(combine_budget, minor_gas_id).await?;

    println!(
        "{} Minting 100,000 NFTs on-chain",
        style("DONE").green().bold()
    );

    Ok(state)
}
