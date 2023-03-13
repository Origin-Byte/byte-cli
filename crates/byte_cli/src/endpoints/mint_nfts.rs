use anyhow::{anyhow, Result};
use console::style;
use gutenberg::Schema;
use rust_sdk::collection_state::{CollectionState, ObjectType};
use std::sync::Arc;
use std::{fs::File, path::PathBuf};
use std::{thread, time};
use terminal_link::Link;
use tokio::task::JoinSet;
use walkdir::WalkDir;

use rust_sdk::{
    mint::{self, NftData},
    utils::{get_active_address, get_client, get_keystore, MoveType},
};

pub async fn mint_nfts(
    schema: &Schema,
    gas_budget: usize,
    metadata_path: PathBuf,
    mut warehouse_id: Option<String>,
    mut state: CollectionState,
) -> Result<CollectionState> {
    let contract_id = Arc::new(schema.contract.as_ref().unwrap().clone());
    println!("Initiliazing process on contract ID: {:?}", contract_id);

    let client = Arc::new(get_client().await.unwrap());
    let keystore = Arc::new(get_keystore().await.unwrap());
    let active_address = get_active_address(&keystore)?;
    let module_name = Arc::new(schema.module_name());
    let gas_budget_ref = Arc::new(gas_budget as u64);
    let mint_cap_arc =
        Arc::new(state.mint_cap.as_ref().unwrap().clone().to_string());

    if warehouse_id.is_none() {
        println!("{} Creating warehouse", style("WIP").cyan().bold());
        let collection_type = MoveType::new(
            schema.contract.as_ref().unwrap().clone(),
            schema.module_name(),
            schema.collection.witness_name(),
        );

        let warehouse_object_id = mint::create_warehouse(
            &client,
            &keystore,
            active_address,
            collection_type,
        )
        .await
        .unwrap();

        warehouse_id = Some(warehouse_object_id.to_string());

        state
            .warehouses
            .push(ObjectType::Warehouse(warehouse_object_id));

        println!("{} Creating warehouse", style("DONE").green().bold());
    }

    let warehouse_id_ref = Arc::new(warehouse_id.unwrap());

    println!("{} Collecting NFT metadata", style("WIP").cyan().bold());
    let mut nft_data_vec: Vec<NftData> = vec![];
    for entry in WalkDir::new(metadata_path) {
        let path = entry.as_ref().unwrap().path();

        if path.is_file() {
            let file = File::open(path)
                .map_err(|_| anyhow!("Couldn't open"))
                .unwrap();

            let nft_data = serde_json::from_reader::<File, NftData>(file)
                .map_err(|_| anyhow!("Couldn't"))
                .unwrap();

            nft_data_vec.push(nft_data);
        }
    }
    println!("{} Collecting NFT metadata", style("DONE").green().bold());

    let mut set = JoinSet::new();
    println!("{} Minting NFTs on-chain", style("WIP").cyan().bold());
    for nft_data in nft_data_vec.drain(..) {
        let ten_millis = time::Duration::from_millis(1000);
        thread::sleep(ten_millis);

        set.spawn(mint::mint_nft(
            client.clone(),
            keystore.clone(),
            nft_data,
            contract_id.clone(),
            warehouse_id_ref.clone(),
            module_name.clone(),
            gas_budget_ref.clone(),
            active_address,
            mint_cap_arc.clone(),
        ));
    }

    while let Some(res) = set.join_next().await {
        res.unwrap().unwrap();
    }

    println!("{} Minting NFTs on-chain", style("DONE").green().bold());
    println!("Warehouse object ID: {warehouse_id_ref}");

    let explorer_link = format!(
        "https://explorer.sui.io/object/{warehouse_id_ref}?network=devnet"
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your NFTs on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(state)
}
