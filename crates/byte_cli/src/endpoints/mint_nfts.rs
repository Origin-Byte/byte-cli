use anyhow::{anyhow, Result};
use gutenberg::Schema;
use std::sync::Arc;
use std::{fs::File, path::PathBuf};
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
    mint_cap: String,
) -> Result<()> {
    let contract_id = Arc::new(schema.contract.as_ref().unwrap().clone());
    println!("Initiliazing process on contract id: {:?}", contract_id);

    let client = Arc::new(get_client().await.unwrap());
    let keystore = Arc::new(get_keystore().await.unwrap());
    let active_address = get_active_address(&keystore)?;
    let module_name = Arc::new(schema.module_name());
    let gas_budget_ref = Arc::new(gas_budget as u64);
    let mint_cap_arc = Arc::new(mint_cap);

    if warehouse_id.is_none() {
        println!("Creating warehouse");
        let collection_type = MoveType::new(
            schema.contract.as_ref().unwrap().clone(),
            schema.module_name(),
            schema.collection.witness_name(),
        );

        warehouse_id = Some(
            mint::create_warehouse(
                &client,
                &keystore,
                active_address,
                collection_type,
            )
            .await
            .unwrap(),
        );
    }
    println!("Built warehouse id: {:?}", warehouse_id);
    let warehouse_id_ref = Arc::new(warehouse_id.unwrap());

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

        let mut set = JoinSet::new();

        for nft_data in nft_data_vec.drain(..) {
            set.spawn(
                mint::handle_mint_nft(
                    client.clone(),
                    keystore.clone(),
                    nft_data,
                    contract_id.clone(),
                    warehouse_id_ref.clone(),
                    module_name.clone(),
                    gas_budget_ref.clone(),
                    active_address,
                    mint_cap_arc.clone(),
                )
                .await,
            );
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap().unwrap();
        }
    }

    Ok(())
}
