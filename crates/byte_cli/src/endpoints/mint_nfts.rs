use anyhow::anyhow;
use gutenberg::Schema;
use std::{fs::File, path::PathBuf};
use walkdir::WalkDir;

use rust_sdk::{
    mint::{self, NftData},
    utils::{get_client, get_keystore},
};

pub async fn mint_nfts(
    schema: &Schema,
    _gas_budget: usize,
    metadata_path: PathBuf,
    mut warehouse_id: Option<String>,
) {
    let contract_id = &schema.contract.as_ref().unwrap();

    let client = get_client().await.unwrap();
    let keystore = get_keystore().await.unwrap();

    if warehouse_id.is_none() {
        warehouse_id =
            Some(mint::create_warehouse(&client, &keystore).await.unwrap());
    }

    for entry in WalkDir::new(metadata_path) {
        let path = entry.as_ref().unwrap().path();

        if path.is_file() {
            let file = File::open(path)
                .map_err(|_| anyhow!("Couldn't open"))
                .unwrap();

            let nft_data = serde_json::from_reader::<File, NftData>(file)
                .map_err(|_| anyhow!("Couldn't"))
                .unwrap();

            mint::mint_nft(
                &client,                                 // sui
                &keystore,                               // keystore
                &nft_data,                               // nft_data
                contract_id.as_str(),                    // package_id
                warehouse_id.as_ref().unwrap().as_str(), // warehouse_id
                "suimarines",                            // module_name
            )
            .await
            .unwrap();
        }
    }
}
