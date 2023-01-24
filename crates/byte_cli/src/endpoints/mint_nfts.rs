use anyhow::anyhow;
use gutenberg::Schema;
use std::{fs::File, path::PathBuf};
use walkdir::WalkDir;

// TODO: Harcode the data here!
use rust_sdk::mint::{self, NftData};

pub async fn mint_nfts(
    schema: &Schema,
    _gas_budget: usize,
    mut config_path: PathBuf,
    mut warehouse_id: Option<String>,
) {
    let contract_id = &schema.contract.as_ref().unwrap();

    // Removes filename from the path
    config_path.pop();

    // Push the folder data, where the json metadata is stored.
    config_path.push("data");

    let client = mint::get_client().await.unwrap();
    let keystore = mint::get_keystore().await.unwrap();

    if warehouse_id.is_none() {
        warehouse_id =
            Some(mint::create_warehouse(&client, &keystore).await.unwrap());
    }

    for entry in WalkDir::new(config_path) {
        // println!("The entry is: {}", entry.unwrap().path().display());

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
                warehouse_id.as_ref().unwrap().as_str(), // module_name
                "suimarines",                            // warehouse_id
            )
            .await
            .unwrap();
        }
    }

    // let args = "suimarines-1",
    // "Suimarine #1",
    // "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/1.png",
    // warehouse_id.as_str(),

    // println!("{}", warehouse_id.unwrap());

    // let _nft_id_2 = mint::mint_nft(
    //     &client,
    //     &keystore,
    //     "suimarines-2",
    //     "Suimarine #2",
    //     "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/2.png",
    //     warehouse_id.as_str(),
    //     contract_id.as_str(),
    // )
    // .await
    // .unwrap();

    // let _nft_id_3 = mint::mint_nft(
    //     &client,
    //     &keystore,
    //     "suimarines-3",
    //     "Suimarine #3",
    //     "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/3.png",
    //     warehouse_id.as_str(),
    //     contract_id.as_str(),
    // )
    // .await
    // .unwrap();

    // let _nft_id_4 = mint::mint_nft(
    //     &client,
    //     &keystore,
    //     "suimarines-4",
    //     "Suimarine #4",
    //     "https://nuno-bucket-1.s3.amazonaws.com/suimarines/images/4.png",
    //     warehouse_id.as_str(),
    //     contract_id.as_str(),
    // )
    // .await
    // .unwrap();
}
