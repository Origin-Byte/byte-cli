use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ini::ini;
use rust_sdk::metadata::GlobalMetadata;
use s3::{bucket::Bucket, creds::Credentials, region::Region};
use serde::{Deserialize, Serialize};
use std::fs;
use std::{path::Path, sync::Arc};
use tokio::task::JoinHandle;

use crate::uploader::{Asset, ParallelUploader, Prepare, UploadedAsset};

// Maximum number of times to retry each individual upload.
const MAX_RETRY: u8 = 3;

#[derive(Debug, Deserialize, Serialize)]
pub struct AWSConfig {
    pub bucket: String,
    pub directory: String,
    pub region: String,
    pub profile: String,
}

impl AWSConfig {
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

pub struct AWSSetup {
    pub bucket: Arc<Bucket>,
    pub directory: String,
    pub url: String,
}

impl AWSSetup {
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

    async fn write(
        bucket: Arc<Bucket>,
        directory: String,
        url: String,
        asset: Asset,
        // mut nft_data: RefMut<'a, u32, Metadata, RandomState>,
        metadata: Arc<GlobalMetadata>,
    ) -> Result<UploadedAsset> {
        println!("Reading from {:?}", asset.path);
        let content = fs::read(&asset.path)?;

        let path = Path::new(&directory).join(&asset.name);
        let path_str = path.to_str().ok_or_else(|| {
            anyhow!("Failed to convert S3 bucket directory path to string.")
        })?;
        let mut retry = MAX_RETRY;

        loop {
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

        Ok(UploadedAsset::new(asset.index, uri.to_string()))
    }
}

#[async_trait]
impl Prepare for AWSSetup {
    async fn prepare(&self, _assets: &[Asset]) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for AWSSetup {
    fn upload_asset(
        &self,
        asset: Asset,
        metadata: Arc<GlobalMetadata>,
    ) -> JoinHandle<Result<UploadedAsset>> {
        let bucket = self.bucket.clone();
        let directory = self.directory.clone();
        let url = self.url.clone();

        // let nft_data = metadata.0.get_mut(&asset.index).unwrap();

        tokio::spawn(async move {
            AWSSetup::write(bucket, directory, url, asset, metadata).await
        })
    }
}

pub fn get_region(profile: &str) -> Result<Region> {
    let region_string = get_region_string(profile)?;

    Ok(region_string.parse()?)
}

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
