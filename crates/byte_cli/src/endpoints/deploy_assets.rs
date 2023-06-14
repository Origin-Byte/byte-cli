use anyhow::Result;
use console::style;
use dotenv::dotenv;
use glob::glob;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use uploader::{
    storage::{
        aws::AWSSetup, nft_storage::NftStorageSetup, pinata::PinataSetup,
    },
    uploader::{Asset, Uploader},
    writer::Storage,
};

pub async fn deploy_assets(
    storage: &Storage,
    assets_dir: PathBuf,
    metadata_dir: PathBuf,
) -> Result<()> {
    let assets_dir = assets_dir.display().to_string();

    dotenv().ok();

    let mut assets: Vec<Asset> = vec![];

    for e in glob(format!("{}*", assets_dir).as_str())
        .expect("Failed to read glob pattern")
    {
        let file_string = e?.file_name().unwrap().to_str().unwrap().to_string();

        let path_string = format!(
            "{assets_dir}{file}",
            assets_dir = assets_dir,
            file = file_string
        );

        let path = Path::new(path_string.as_str());
        let file_name = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().and_then(OsStr::to_str);

        if extension.is_none() {
            println!("Skipping File: {}", file_name);
            continue;
        }

        let mut content_type = String::from("image/");
        // Can safely unwrap as we have asserted that !is_none
        content_type.push_str(extension.unwrap());

        let asset = Asset::new(
            String::from(file_name),
            file_string,
            PathBuf::from(path),
            content_type, // MIME content type
        );

        assets.push(asset);
    }

    if assets.is_empty() {
        panic!("Assets folder is empty. Make sure that you are in the right project folder and that you have your images in the assets/ folder within it.");
    }

    println!("{} Uploading images to storage", style("WIP").cyan().bold());
    match storage {
        Storage::Aws(config) => {
            let setup = AWSSetup::new(config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            uploader.upload(&mut assets, metadata_dir, false).await?;
        }
        Storage::Pinata(config) => {
            let setup = PinataSetup::new(config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            uploader.prepare(&assets).await?;

            uploader.upload(&mut assets, metadata_dir, false).await?;
        }
        Storage::NftStorage(config) => {
            let setup = NftStorageSetup::new(config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            uploader.prepare(&assets).await?;

            uploader.upload(&mut assets, metadata_dir, false).await?;
        }
    }
    println!(
        "{} Uploading images to storage",
        style("DONE").green().bold()
    );

    Ok(())
}
