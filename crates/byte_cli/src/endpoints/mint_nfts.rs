use anyhow::Result;
use console::style;
use rust_sdk::coin;
use rust_sdk::collection_state::CollectionState;
use std::sync::Arc;
use std::{thread, time};
use sui_sdk::types::base_types::ObjectID;
use tokio::task::JoinSet;

use rust_sdk::{
    mint::{self},
    utils::{get_active_address, get_context},
};

pub async fn mint_nfts(
    // schema: &Schema,
    gas_budget: usize,
    // metadata_path: PathBuf,
    // mut warehouse_id: Option<String>,
    state: CollectionState,
) -> Result<CollectionState> {
    let contract_id = Arc::new(state.contract.as_ref().unwrap().to_string());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let wallet_ctx = Arc::new(get_context().await.unwrap());
    let active_address =
        get_active_address(&wallet_ctx.config.keystore).unwrap();

    let module_name = Arc::new(String::from("xmachina"));
    // let module_name = Arc::new(schema.package_name());
    let gas_budget_ref = Arc::new(gas_budget as u64);

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
                wallet_ctx.clone(),
                contract_id.clone(),
                module_name.clone(),
                gas_budget_ref.clone(),
                active_address,
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
    project_name: String,
    gas_budget: usize,
    state: CollectionState,
    main_gas_id: ObjectID,
    minor_gas_id: ObjectID,
) -> Result<CollectionState> {
    let contract_id = Arc::new(state.contract.as_ref().unwrap().to_string());
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
