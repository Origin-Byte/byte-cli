use anyhow::Result;
use console::style;
// use gutenberg::Schema;
use rust_sdk::collection_state::CollectionState;
use std::sync::Arc;
// use std::{fs::File, path::PathBuf};
use std::{thread, time};
// use terminal_link::Link;
use tokio::task::JoinSet;
// use walkdir::WalkDir;

use rust_sdk::{
    mint::{self},
    utils::{get_active_address, get_client, get_keystore},
};

pub async fn mint_nfts(
    // schema: &Schema,
    gas_budget: usize,
    // metadata_path: PathBuf,
    // mut warehouse_id: Option<String>,
    state: CollectionState,
) -> Result<CollectionState> {
    let contract_id =
        Arc::new(state.contract.as_ref().unwrap().clone().to_string());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    // let client = Arc::new(get_client().await.unwrap());
    let keystore = Arc::new(get_keystore().await.unwrap());
    let active_address = get_active_address(&keystore)?;
    let module_name = Arc::new(String::from("xmachina"));
    // let module_name = Arc::new(schema.package_name());
    let gas_budget_ref = Arc::new(gas_budget as u64);
    // let mint_cap_arc =
    //     Arc::new(state.mint_cap.as_ref().unwrap().clone().to_string());

    // if warehouse_id.is_none() {
    //     println!("{} Creating warehouse", style("WIP").cyan().bold());
    //     let collection_type = MoveType::new(
    //         state.contract.as_ref().unwrap().clone().to_string(),
    //         schema.package_name(),
    //         schema.collection.witness_name(),
    //     );

    //     let warehouse_object_id =
    //         mint::create_warehouse(&client, active_address, collection_type)
    //             .await
    //             .unwrap();

    //     warehouse_id = Some(warehouse_object_id.to_string());

    //     state
    //         .warehouses
    //         .push(ObjectType::Warehouse(warehouse_object_id));

    //     println!("{} Creating warehouse", style("DONE").green().bold());
    // }

    // let warehouse_id_ref = Arc::new(warehouse_id.unwrap());

    println!("{} Collecting NFT metadata", style("WIP").cyan().bold());
    // let mut nft_data_vec: Vec<NftData> = vec![];
    // for entry in WalkDir::new(metadata_path) {
    //     let path = entry.as_ref().unwrap().path();

    //     if path.is_file() {
    //         let file = File::open(path)
    //             .map_err(|_| anyhow!("Couldn't open"))
    //             .unwrap();

    //         let nft_data = serde_json::from_reader::<File, NftData>(file)
    //             .map_err(|_| anyhow!("Couldn't"))
    //             .unwrap();

    //         nft_data_vec.push(nft_data);
    //     }
    // }
    println!("{} Collecting NFT metadata", style("DONE").green().bold());

    let mut set = JoinSet::new();
    println!("{} Minting NFTs on-chain", style("WIP").cyan().bold());
    // for nft_data in nft_data_vec.drain(..) {
    //     let ten_millis = time::Duration::from_millis(1000);
    //     thread::sleep(ten_millis);

    //     set.spawn(
    //         mint::handle_mint_nft(
    //             client.clone(),
    //             keystore.clone(),
    //             // nft_data,
    //             contract_id.clone(),
    //             // warehouse_id_ref.clone(),
    //             module_name.clone(),
    //             gas_budget_ref.clone(),
    //             active_address,
    //             // mint_cap_arc.clone(),
    //         )
    //         .await,
    //     );
    // }
    for _i in 0..10 {
        set.spawn(
            mint::handle_mint_nft(
                // client.clone(),
                keystore.clone(),
                // nft_data,
                contract_id.clone(),
                // warehouse_id_ref.clone(),
                module_name.clone(),
                gas_budget_ref.clone(),
                active_address,
                // mint_cap_arc.clone(),
            )
            .await,
        );

        let ten_millis = time::Duration::from_millis(2000);
        thread::sleep(ten_millis);
    }

    while let Some(res) = set.join_next().await {
        res.unwrap().unwrap().unwrap();
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
    // schema: &Schema,
    gas_budget: usize,
    // metadata_path: PathBuf,
    // mut warehouse_id: Option<String>,
    state: CollectionState,
) -> Result<CollectionState> {
    let contract_id =
        Arc::new(state.contract.as_ref().unwrap().clone().to_string());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let client = Arc::new(get_client().await.unwrap());
    let keystore = Arc::new(get_keystore().await.unwrap());
    let active_address = get_active_address(&keystore)?;
    let module_name = Arc::new(String::from("xmachina"));
    // let module_name = Arc::new(schema.package_name());
    let gas_budget_ref = Arc::new(gas_budget as u64);
    // let mint_cap_arc =
    //     Arc::new(state.mint_cap.as_ref().unwrap().clone().to_string());

    // if warehouse_id.is_none() {
    //     println!("{} Creating warehouse", style("WIP").cyan().bold());
    //     let collection_type = MoveType::new(
    //         state.contract.as_ref().unwrap().clone().to_string(),
    //         schema.package_name(),
    //         schema.collection.witness_name(),
    //     );

    //     let warehouse_object_id =
    //         mint::create_warehouse(&client, active_address, collection_type)
    //             .await
    //             .unwrap();

    //     warehouse_id = Some(warehouse_object_id.to_string());

    //     state
    //         .warehouses
    //         .push(ObjectType::Warehouse(warehouse_object_id));

    //     println!("{} Creating warehouse", style("DONE").green().bold());
    // }

    // let warehouse_id_ref = Arc::new(warehouse_id.unwrap());

    println!("{} Collecting NFT metadata", style("WIP").cyan().bold());
    // let mut nft_data_vec: Vec<NftData> = vec![];
    // for entry in WalkDir::new(metadata_path) {
    //     let path = entry.as_ref().unwrap().path();

    //     if path.is_file() {
    //         let file = File::open(path)
    //             .map_err(|_| anyhow!("Couldn't open"))
    //             .unwrap();

    //         let nft_data = serde_json::from_reader::<File, NftData>(file)
    //             .map_err(|_| anyhow!("Couldn't"))
    //             .unwrap();

    //         nft_data_vec.push(nft_data);
    //     }
    // }
    println!("{} Collecting NFT metadata", style("DONE").green().bold());

    let mut set = JoinSet::new();
    println!(
        "{} Minting 100,000 NFTs on-chain",
        style("WIP").cyan().bold()
    );

    // for nft_data in nft_data_vec.drain(..) {
    //     let ten_millis = time::Duration::from_millis(1000);
    //     thread::sleep(ten_millis);

    //     set.spawn(
    //         mint::handle_mint_nft(
    //             client.clone(),
    //             keystore.clone(),
    //             // nft_data,
    //             contract_id.clone(),
    //             // warehouse_id_ref.clone(),
    //             module_name.clone(),
    //             gas_budget_ref.clone(),
    //             active_address,
    //             // mint_cap_arc.clone(),
    //         )
    //         .await,
    //     );
    // }
    let split = 100;

    mint::split(None, split, 500000000 as u64).await?;

    let (_, mut coins_to_merge) =
        mint::get_coin_separated(&client, active_address).await?;

    assert!(coins_to_merge.len() == split as usize);

    let mut j = 0;

    for i in 0..split {
        let gas_coin = coins_to_merge.pop().unwrap();
        let gas_coin_ref =
            (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest);

        set.spawn(
            mint::handle_parallel_mint_nft(
                // client.clone(),
                keystore.clone(),
                // nft_data,
                contract_id.clone(),
                // warehouse_id_ref.clone(),
                module_name.clone(),
                gas_budget_ref.clone(),
                Arc::new(gas_coin_ref),
                active_address,
                // i as usize,
                // mint_cap_arc.clone(),
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

    mint::combine(500000000 as u64).await?;

    println!(
        "{} Minting 100,000 NFTs on-chain",
        style("DONE").green().bold()
    );

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
