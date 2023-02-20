use anyhow::{anyhow, Result};
use async_trait::async_trait;
use ini::ini;
use s3::{bucket::Bucket, creds::Credentials, region::Region};
use serde::{Deserialize, Serialize};
use std::fs;
use std::{path::Path, str::FromStr, sync::Arc};
use std::{thread, time};
use tokio::sync::mpsc::Sender;
use tokio::task::{AbortHandle, JoinHandle, JoinSet};

use crate::storage::uploader::Asset;

use super::{ParallelUploader, Prepare, UploadedAsset};

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
        region: String,
        profile: String,
    ) -> Self {
        Self {
            bucket,
            directory,
            region,
            profile,
        }
    }

    pub fn new_from_profile() -> Self {
        todo!()
    }

    pub fn get_region(profile: &str) -> Result<Region> {
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

        Ok(region.parse()?)
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

        let region = Region::from_str(config.region.as_str())?;

        let bucket = Bucket::new(&config.bucket, region, credentials)?;
        let url = format!(
            "https://{}.s3.amazonaws.com/{}",
            config.bucket, config.directory
        );

        // TODO: Assert that S3 Bucket region exists

        Ok(Self {
            bucket: Arc::new(bucket),
            directory: config.directory.to_string(),
            url: format!(
                "https://{}.s3.amazonaws.com/{}",
                config.bucket, config.directory
            ),
        })
    }

    async fn write(
        // tx: Sender<UploadedAsset>,
        bucket: Arc<Bucket>,
        directory: String,
        url: String,
        asset: Asset,
    ) -> Result<UploadedAsset> {
        println!("DEBUG: Uploading asset on AWS");
        println!("DEBUG: The asset path I am reading from: {:?}", asset.path);

        println!("DEBUG: The bucket is the following: {:?}", bucket);

        // let content: Vec<u8> = vec![];
        let content = fs::read(&asset.path)?;

        let path = Path::new(&directory).join(&asset.id);
        let path_str = path.to_str().ok_or_else(|| {
            anyhow!("Failed to convert S3 bucket directory path to string.")
        })?;
        let mut retry = MAX_RETRY;
        println!("DEBUG: Putting object with content type: path -> {:?}, type -> {:?}", path_str, asset.content_type,);

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
                        println!("DEBUG: AWS returned ok with code {:}", code);
                        break;
                    }
                    _ => {
                        println!("DEBUG: AWS returned err with code {:}", code);
                        return Err(anyhow!(
                            "Failed to upload {} to S3 with Http Code: {code}",
                            asset.id
                        ));
                    }
                },
                Err(error) => {
                    println!("DEBUG: AWS returned err with code {:?}", error);
                    if retry == 0 {
                        return Err(error.into());
                    }
                    // we try one more time before reporting the error
                    retry -= 1;
                }
            }
        }

        let link = url::Url::parse(&url)?.join("/")?.join(path_str)?;

        println!("Successfully uploaded {} to S3 at {}", asset.id, link);

        Ok(UploadedAsset::new(asset.id.clone(), link.to_string()))
    }
}

#[async_trait]
impl Prepare for AWSSetup {
    async fn prepare(&self, _assets: &Vec<Asset>) -> Result<()> {
        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for AWSSetup {
    fn upload_asset(
        &self,
        // set: &mut JoinSet<()>,
        // tx: Sender<UploadedAsset>,
        asset: Asset,
    ) -> JoinHandle<Result<UploadedAsset>> {
        let bucket = self.bucket.clone();
        let directory = self.directory.clone();
        let url = self.url.clone();

        tokio::spawn(async move {
            AWSSetup::write(bucket, directory, url, asset).await
        })
    }
}
