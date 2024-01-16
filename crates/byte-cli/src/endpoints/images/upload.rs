use crate::io::{write_json, LocalWrite};
use anyhow::{anyhow, Result};
use byte::io::LocalRead;
use chrono::Local;
use console::style;
use dotenv::dotenv;
use glob::glob;
use rust_sdk::metadata::{GlobalMetadata, StorableMetadata};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use uploader::{
    storage::{aws::AWSSetup, pinata::PinataSetup},
    uploader::{Asset, Uploader},
    writer::Storage,
};

/// Deploys assets to a specified storage service.
///
/// # Arguments
/// * `storage` - A reference to the Storage configuration.
/// * `assets_dir` - PathBuf to the directory containing assets.
/// * `pre_upload_path` - PathBuf to the pre-upload metadata JSON file.
/// * `post_upload_path` - PathBuf to the post-upload metadata JSON file.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
///
/// # Functionality
/// - Prepares the assets directory and reads the metadata file.
/// - Creates error logs and asset structures for each image file.
/// - Skips files already uploaded and those not matching expected formats.
/// - Uploads the assets to the specified storage (AWS or Pinata).
/// - Writes error logs and updates the metadata post-upload.
/// - Prints a summary of the upload process.
pub async fn deploy_assets(
    storage: &Storage,
    assets_dir: PathBuf,
    pre_upload_path: PathBuf,
    post_upload_path: PathBuf,
) -> Result<()> {
    fs::create_dir_all(assets_dir.clone())?;
    let assets_dir = assets_dir.display().to_string();

    dotenv().ok();

    let storable_meta = StorableMetadata::read_json(
        if Path::new(&post_upload_path).exists() {
            // If post upload path exists, then it means that the upload
            // has already started in the past and that most likely the process
            // is pending
            &post_upload_path
        } else {
            // If post upload file does not exist, then start from scratch
            if !Path::new(&pre_upload_path).exists() {
                // Creates the metadata folder if it does not exist
                fs::create_dir_all(pre_upload_path.parent().unwrap())?;

                return Err(anyhow!(format!(
                    "Unable to upload assets without the metadata file, used to store the URLs. Make sure you add the NFT images to {} create and the metadata to {:?}", assets_dir, pre_upload_path
                )));
            }

            &pre_upload_path
        },
    )?;

    let metadata_idxs = storable_meta.get_to_upload();
    let shared_metadata = Arc::new(GlobalMetadata::from_map(storable_meta));

    let mut error_path = pre_upload_path.parent().unwrap().to_path_buf();
    let now = Local::now().format("%Y%m%d%H%M%S").to_string();
    error_path.push(format!("logs/upload-{}.json", now));

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

        let index: u32 = file_name.parse().expect(
            format!("Failed to parse file name {:?} as an integer", file_name)
                .as_str(),
        );

        if !metadata_idxs.contains(&index) {
            println!("Skipping File: {}. Already uploaded", file_name);
            continue;
        }

        let asset = Asset::new(
            index,
            file_string,
            PathBuf::from(path),
            content_type, // MIME content type
        );

        assets.push(asset);
    }

    if assets.is_empty() {
        return Err(anyhow!(format!("Could not find images to upload. Happens either if no images in the folder {} or if all images have already been uploaded", assets_dir)));
    }

    let jobs_no = assets.len();

    println!("{} Uploading images to storage", style("WIP").cyan().bold());
    let error_logs = match storage {
        Storage::Aws(config) => {
            let setup = AWSSetup::new(config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            uploader
                .upload(&mut assets, shared_metadata.clone())
                .await?
        }
        Storage::Pinata(config) => {
            let setup = PinataSetup::new(config).await?;
            let uploader = Box::new(setup) as Box<dyn Uploader>;

            uploader.prepare(&assets).await?;

            uploader
                .upload(&mut assets, shared_metadata.clone())
                .await?
        } // TODO: Add back
          // Storage::NftStorage(config) => {
          //     let setup = NftStorageSetup::new(config).await?;
          //     let uploader = Box::new(setup) as Box<dyn Uploader>;

          //     uploader.prepare(&assets).await?;

          //     uploader
          //         .upload(&mut assets, shared_metadata.clone())
          //         .await?
          // }
    };

    let failed_jobs = error_logs.len();

    if !error_logs.is_empty() {
        write_json(error_logs, error_path.as_path())?;
    }

    let dash_map = Arc::try_unwrap(shared_metadata)
        .map_err(|_| anyhow!("Failed to unwrap Arc"))?;

    let map = StorableMetadata::from_map(dash_map.into_map());

    map.write_json(post_upload_path.as_path())?;

    println!(
        "{} Uploading images to storage",
        style("DONE").green().bold()
    );

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

    Ok(())
}
