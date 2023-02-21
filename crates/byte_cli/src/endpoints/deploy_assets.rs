use anyhow::Result;
use dotenv::dotenv;
use glob::glob;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use gutenberg::storage::{
    aws::AWSSetup, Asset, Storage, StorageState, Uploader,
};
use gutenberg::Schema;

pub async fn deploy_assets(schema: &Schema, assets_dir: PathBuf) -> Result<()> {
    let assets_dir = assets_dir.display().to_string();
    let storage = schema.storage.as_ref().unwrap();

    dotenv().ok();

    println!("Storage Policy: {:?}", storage);

    let mut files: Vec<String> = vec![];

    for e in glob(format!("{}*", assets_dir).as_str())
        .expect("Failed to read glob pattern")
    {
        let file_string = e?.file_name().unwrap().to_str().unwrap().to_string();

        files.push(file_string);
    }

    if files.is_empty() {
        panic!("Assets folder is empty. Make sure that you are in the right project folder and that you have your images in the assets/ folder within it.");
    }

    let mut assets: Vec<Asset> = vec![];

    for file in files {
        // TODO: Make this multi-threaded
        let path_string = format!(
            "{assets_dir}/{file}",
            assets_dir = assets_dir,
            file = file
        );

        let path = Path::new(path_string.as_str());

        let file_name = path.file_stem().unwrap().to_str().unwrap();

        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .expect("Failed to convert extension from unicode");

        let asset = Asset::new(
            String::from(file_name),
            PathBuf::from(path),
            String::from(extension),
        );

        println!("File: {:?}", asset);
        assets.push(asset);

        // aws::upload_file(&client, bucket_name.as_str(), path).await?;
    }

    let mut storage_state = StorageState::default();

    match storage {
        Storage::Aws(config) => {
            let setup = AWSSetup::new(&config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            // TODO: Missing uploader prepare

            uploader
                .upload(&mut assets, &mut storage_state, false)
                .await?;
        }
        Storage::Pinata(_config) => {}
        Storage::NftStorage(_config) => {}
    }

    Ok(())
}
