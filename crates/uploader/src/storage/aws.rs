use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ini::ini;
use rust_sdk::metadata::GlobalMetadata;
use s3::{bucket::Bucket, creds::Credentials, region::Region};
use serde::{Deserialize, Serialize};
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{path::Path, sync::Arc};
use tokio::task::JoinHandle;

use crate::uploader::{
    Asset, ParallelUploader, Prepare, UploadEffects, UploadedAsset,
};

// Maximum number of times to retry each individual upload.
const MAX_RETRY: u8 = 1;

/// Configuration for AWS services, including bucket, directory, and region.
#[derive(Debug, Deserialize, Serialize)]
pub struct AWSConfig {
    pub bucket: String,
    pub directory: String,
    pub region: String,
    pub profile: String,
}

impl AWSConfig {
    /// Creates a new `AWSConfig`.
    ///
    /// # Arguments
    /// * `bucket` - The S3 bucket name.
    /// * `directory` - The directory within the bucket.
    /// * `region` - The AWS region.
    /// * `profile` - The AWS profile to use.
    ///
    /// # Returns
    /// A result containing the new AWSConfig or an error.
    pub fn new(
        bucket: String,
        directory: String,
        mut region: String,
        profile: String,
    ) -> Result<Self> {
        assert_profile(profile.as_str())?;

        if region == "default" {
            region = get_region_string(profile.as_str())?;
        }

        Ok(Self {
            bucket,
            directory,
            region,
            profile,
        })
    }
}

/// Setup for AWS services, encapsulating bucket and directory information.
pub struct AWSSetup {
    pub bucket: Arc<Bucket>,
    pub directory: String,
    pub url: String,
}

impl AWSSetup {
    /// Initializes a new `AWSSetup` based on provided AWS configuration.
    ///
    /// # Arguments
    /// * `config` - The AWS configuration.
    ///
    /// # Returns
    /// A result containing the new `AWSSetup` or an error.
    pub async fn new(config: &AWSConfig) -> Result<Self> {
        let credentials =
            Credentials::from_profile(Some(config.profile.as_str()))?;

        let region = config.region.parse()?;

        if let Region::Custom {
            region,
            endpoint: _,
        } = region
        {
            return Err(anyhow!("Custom AWS region {} not supported", region));
        }

        Ok(Self {
            bucket: Arc::new(Bucket::new(&config.bucket, region, credentials)?),
            directory: config.directory.to_string(),
            url: format!(
                "https://{}.s3.amazonaws.com/{}",
                config.bucket, config.directory
            ),
        })
    }

    /// Asynchronously writes an asset to AWS S3.
    ///
    /// # Arguments
    /// * `bucket` - The AWS S3 bucket.
    /// * `directory` - The directory within the bucket.
    /// * `url` - The base URL of the AWS bucket.
    /// * `asset` - The asset to upload.
    /// * `metadata` - The global metadata.
    /// * `terminate_flag` - Flag indicating if the upload process should be
    ///   terminated.
    ///
    /// # Returns
    /// A result indicating the success or failure of the upload.
    async fn write(
        bucket: Arc<Bucket>,
        directory: String,
        url: String,
        asset: Asset,
        metadata: Arc<GlobalMetadata>,
        terminate_flag: Arc<AtomicBool>,
    ) -> Result<UploadEffects> {
        let content = fs::read(&asset.path)?;

        let path = Path::new(&directory).join(&asset.name);
        let path_str = path.to_str().ok_or_else(|| {
            anyhow!("Failed to convert S3 bucket directory path to string.")
        })?;
        let mut retry = MAX_RETRY;

        loop {
            // Note: Here for testing
            tokio::time::sleep(Duration::from_millis(10 * asset.index as u64))
                .await;

            if terminate_flag.load(Ordering::SeqCst) {
                // Terminate the loop if terminate_flag is true
                return Ok(UploadEffects::Failure(asset.index));
            }

            match bucket
                .put_object_with_content_type(
                    path_str,
                    &content,
                    &asset.content_type,
                )
                .await
            {
                Ok((_, code)) => match code {
                    200 => {
                        break;
                    }
                    _ => {
                        return Err(anyhow!(
                            "Failed to upload {} to S3 with Http Code: {code}",
                            asset.index
                        ));
                    }
                },
                Err(error) => {
                    if retry == 0 {
                        return Err(error.into());
                    }
                    // we try one more time before reporting the error
                    retry -= 1;
                }
            }
        }

        let uri = url::Url::parse(&url)?.join("/")?.join(path_str)?;

        // Access the url field on the Metadata struct
        let mut nft_data = metadata.0.get_mut(&asset.index).unwrap();
        nft_data.url = Some(uri.clone());

        Ok(UploadEffects::Success(UploadedAsset::new(
            asset.index,
            uri.to_string(),
        )))
    }
}

/// Implementation of the `Prepare` trait for `AWSSetup`.
#[async_trait]
impl Prepare for AWSSetup {
    async fn prepare(&self, _assets: &[Asset]) -> Result<()> {
        Ok(())
    }
}

/// Implementation of `ParallelUploader` for `AWSSetup`.
#[async_trait]
impl ParallelUploader for AWSSetup {
    /// Uploads an asset to AWS S3 in a parallel manner.
    ///
    /// # Arguments
    /// * `asset` - The asset to be uploaded.
    /// * `metadata` - The global metadata.
    /// * `terminate_flag` - Flag indicating if the upload process should be
    ///   terminated.
    ///
    /// # Returns
    /// A `JoinHandle` that resolves to a result of `UploadEffects`.
    fn upload_asset(
        &self,
        asset: Asset,
        metadata: Arc<GlobalMetadata>,
        terminate_flag: Arc<AtomicBool>,
    ) -> JoinHandle<Result<UploadEffects>> {
        let bucket = self.bucket.clone();
        let directory = self.directory.clone();
        let url = self.url.clone();

        tokio::spawn(async move {
            AWSSetup::write(
                bucket,
                directory,
                url,
                asset,
                metadata,
                terminate_flag,
            )
            .await
        })
    }
}

/// Fetches the AWS region from the user's AWS configuration.
///
/// # Arguments
/// * `profile` - The AWS profile name.
///
/// # Returns
/// A result containing the AWS `Region` or an error.
pub fn get_region(profile: &str) -> Result<Region> {
    let region_string = get_region_string(profile)?;

    Ok(region_string.parse()?)
}

/// Retrieves the region string from the AWS configuration file based on the
/// profile.
///
/// # Arguments
/// * `profile` - The AWS profile name.
///
/// # Returns
/// A result containing the region string or an error.
pub fn get_region_string(profile: &str) -> Result<String> {
    let home_dir = dirs::home_dir()
        .expect("Unexpected error: Could not find home directory.");
    let config_path = home_dir.join(Path::new(".aws/config"));
    let aws_config = ini!(config_path
        .to_str()
        .ok_or_else(|| anyhow!("Could not load AWS config file. Make sure you have AWS CLI installed and a profile configured locally"))?);

    let region = &aws_config
        .get(profile)
        .ok_or_else(|| {
            anyhow!("Profile not found in AWS config file! Make sure you have AWS CLI installed and a profile configured locally")
        })?
        .get("region")
        .ok_or_else(|| {
            anyhow!("Region not found in AWS config file!")
        })?
        .as_ref()
        .ok_or_else(|| {
            anyhow!("Unexpected error: Failed while fetching AWS region from config file.")
        })?
        .to_string();

    Ok(region.clone())
}

/// Asserts the existence of a specified AWS profile in the AWS configuration
/// file.
///
/// # Arguments
/// * `profile` - The AWS profile name to be validated.
///
/// # Returns
/// A result indicating success or an error if the profile is not found.
pub fn assert_profile(profile: &str) -> Result<()> {
    let home_dir = dirs::home_dir()
        .expect("Unexpected error: Could not find home directory.");
    let config_path = home_dir.join(Path::new(".aws/config"));
    let aws_config = ini!(config_path
        .to_str()
        .ok_or_else(|| anyhow!("Could not load AWS config file. Make sure you have AWS CLI installed and a profile configured locally"))?);

    if !aws_config.contains_key(profile) {
        Err(anyhow!("Profile not found in AWS config file! Make sure you have AWS CLI installed and a profile configured locally"))
    } else {
        Ok(())
    }
}
