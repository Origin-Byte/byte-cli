use anyhow::{anyhow, Result};
use async_trait::async_trait;

use reqwest::{
    header,
    multipart::{Form, Part},
    Client, StatusCode,
};
use rust_sdk::metadata::GlobalMetadata;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    sync::atomic::{AtomicBool, Ordering},
};
use std::{path::Path, sync::Arc};
use tokio::task::JoinHandle;

use crate::uploader::{
    Asset, ParallelUploader, Prepare, UploadEffects, UploadedAsset,
};

// For more check: https://docs.pinata.cloud/pinata-api/pinning
const UPLOAD_ENDPOINT: &str = "/pinning/pinFileToIPFS";
// For more check: https://docs.pinata.cloud/pinata-api/authentication
const TEST_AUTH_ENDPOINT: &str =
    "https://api.pinata.cloud/data/testAuthentication";

// File size limit is 10MB
const FILE_SIZE_LIMIT: u64 = 10 * 1024 * 1024;

fn default_limit() -> u16 {
    45
}

/// response after an nft was stored
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    /// Pinata CID - IPFS Hash
    pub ipfs_hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PinataConfig {
    pub jwt: String,
    pub upload_gateway: String,
    pub retrieval_gateway: String,
    #[serde(default = "default_limit")]
    pub parallel_limit: u16,
}

impl PinataConfig {
    pub fn new(
        jwt: String,
        upload_gateway: String,
        retrieval_gateway: String,
        parallel_limit: u16,
    ) -> Self {
        Self {
            jwt,
            upload_gateway,
            retrieval_gateway,
            parallel_limit,
        }
    }
}

pub struct Setup {
    pub client: Client,
    pub endpoint: url::Url,
    pub content_gateway: String,
    pub retrieval_gateway: String,
    pub parallel_limit: u16,
}

pub struct PinataSetup(Arc<Setup>);

impl PinataSetup {
    pub async fn new(config: &PinataConfig) -> Result<Self> {
        let client_builder = Client::builder();

        let mut headers = header::HeaderMap::new();
        let bearer_value = format!("Bearer {}", &config.jwt);
        let mut auth_value = header::HeaderValue::from_str(&bearer_value)?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = client_builder.default_headers(headers).build()?;

        // Testing authentication with client setup in TestAuth endpoint
        let response = client.get(TEST_AUTH_ENDPOINT).send().await?;

        match response.status() {
            StatusCode::OK => {
                // Upload endpoint
                let endpoint = url::Url::parse(&config.upload_gateway)?
                    .join(UPLOAD_ENDPOINT)?;

                Ok(Self(Arc::new(Setup {
                    client,
                    endpoint,
                    content_gateway: config.upload_gateway.clone(),
                    retrieval_gateway: config.retrieval_gateway.clone(),
                    parallel_limit: config.parallel_limit,
                })))
            }
            StatusCode::UNAUTHORIZED => {
                Err(anyhow!("Invalid pinata JWT token."))
            }
            other_codes => Err(anyhow!(
                "Error while initializing pinata client: {other_codes}"
            )),
        }
    }

    async fn write<'a>(
        setup: &Setup,
        asset: Asset,
        // mut nft_data: RefMut<'a, u32, Metadata, RandomState>,
        metadata: Arc<GlobalMetadata>,
        terminate_flag: Arc<AtomicBool>,
    ) -> Result<UploadEffects> {
        // TODO: Each uplaod is creating their own CID, however, this
        // should be shared accross the collection..
        let content = fs::read(&asset.path)?;

        let mut form = Form::new();

        let file = Part::bytes(content)
            .file_name(asset.name.clone())
            .mime_str(asset.content_type.as_str())?;

        form = form
            .part("file", file)
            .text("pinataOptions", "{\"wrapWithDirectory\": true}");

        // Note: Here for testing
        // tokio::time::sleep(Duration::from_millis(100 * asset.index as u64)).await;

        if terminate_flag.load(Ordering::SeqCst) {
            // Terminate the loop if terminate_flag is true
            return Ok(UploadEffects::Failure(asset.index));
        }

        let response = setup
            .client
            .post(&setup.endpoint.to_string())
            .multipart(form)
            .send()
            .await?;

        let status = response.status();

        if status.is_success() {
            let body = response.json::<serde_json::Value>().await?;
            let Response { ipfs_hash } = serde_json::from_value(body)?;

            let uri = url::Url::parse(&setup.retrieval_gateway)?
                .join(&format!("/ipfs/{}/{}", ipfs_hash, asset.name))?;

            // Access the url field on the Metadata struct
            let mut nft_data = metadata.0.get_mut(&asset.index).unwrap();
            nft_data.url = Some(uri.clone());

            Ok(UploadEffects::Success(UploadedAsset::new(
                asset.index,
                uri.to_string(),
            )))
        } else {
            let body = response.json::<serde_json::Value>().await?;

            Err(anyhow!(format!(
                "Error uploading batch with status ({}): {}",
                status, body
            )))
        }
    }
}

#[async_trait]
impl Prepare for PinataSetup {
    async fn prepare(&self, assets: &[Asset]) -> Result<()> {
        assets.iter().try_for_each(|asset| {
            let size = {
                let path = Path::new(&asset.path);
                fs::metadata(path)
            }?
            .len();

            if size > FILE_SIZE_LIMIT {
                return Err(anyhow!(
                    "File '{}' exceeds the 10MB file size limit",
                    asset.path.as_path().display(),
                ));
            }
            Ok(())
        })?;

        Ok(())
    }
}

#[async_trait]
impl ParallelUploader for PinataSetup {
    fn upload_asset(
        &self,
        asset: Asset,
        metadata: Arc<GlobalMetadata>,
        terminate_flag: Arc<AtomicBool>,
        // nft_data: RefMut<u32, Metadata, RandomState>,
    ) -> JoinHandle<Result<UploadEffects>> {
        let setup = self.0.clone();

        tokio::spawn(async move {
            PinataSetup::write(&setup, asset, metadata, terminate_flag).await
        })
    }
}
