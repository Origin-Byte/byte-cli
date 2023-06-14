use anyhow::{anyhow, Result};
use std::{fs, path::Path, sync::Arc};

use async_trait::async_trait;
use reqwest::{
    header,
    multipart::{Form, Part},
    Client, StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};

use crate::uploader::{Asset, Prepare, Uploader};
use std::path::PathBuf;

const NFT_STORAGE_API_URL: &str = "https://api.nft.storage";
// const NFT_STORAGE_GATEWAY_URL: &str = "https://nftstorage.link/ipfs";
// Request time window in (ms) to avoid the rate limit
const REQUEST_WAIT: u64 = 10000;
// File size limit is 10MB
const FILE_SIZE_LIMIT: u64 = 100 * 1024 * 1024;
// Number of files per request limit
const FILE_COUNT_LIMIT: u64 = 100;

#[derive(Debug, Deserialize, Default)]
pub struct StoreNftResponse {
    /// status of the request
    pub ok: bool,
    /// stored nft data
    pub value: Cid,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Cid {
    /// ipfs cid (file hash)
    pub cid: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NftStorageConfig {
    pub auth_token: String,
}

impl NftStorageConfig {
    pub fn new(auth_token: String) -> Self {
        Self { auth_token }
    }
}

pub struct NftStorageSetup {
    client: Arc<Client>,
}

impl NftStorageSetup {
    pub async fn new(config: &NftStorageConfig) -> Result<Self> {
        let client_builder = Client::builder();

        let mut headers = header::HeaderMap::new();
        let bearer_value = format!("Bearer {}", config.auth_token);
        let mut auth_value = header::HeaderValue::from_str(&bearer_value)?;
        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);

        let client = client_builder.default_headers(headers).build()?;

        let url = format!("{}/", NFT_STORAGE_API_URL);
        let response = client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => Ok(Self {
                client: Arc::new(client),
            }),
            StatusCode::UNAUTHORIZED => {
                Err(anyhow!("Invalid nft.storage authentication token."))
            }
            other_codes => Err(anyhow!(
                "Error while initializing nft.storage client: {other_codes}"
            )),
        }
    }
}

#[async_trait]
impl Prepare for NftStorageSetup {
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
impl Uploader for NftStorageSetup {
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        _state: PathBuf,
        _lazy: bool,
    ) -> Result<()> {
        // TODO: Write state to metadata objects
        println!("We are in the upload phase");
        let mut batches: Vec<Vec<&Asset>> = Vec::new();
        let mut current: Vec<&Asset> = Vec::new();
        let mut upload_size = 0;
        let mut upload_count = 0;

        // Make this iteration functional instead
        for asset in assets {
            let size = {
                let path = Path::new(&asset.path);
                fs::metadata(path)
            }?
            .len();

            if (upload_size + size) > FILE_SIZE_LIMIT
                || (upload_count + 1) > FILE_COUNT_LIMIT
            {
                batches.push(current);
                current = Vec::new();
                upload_size = 0;
                upload_count = 0;
            }

            upload_size += size;
            upload_count += 1;
            current.push(asset);
        }
        // adds the last chunk if there is one
        if !current.is_empty() {
            batches.push(current);
        }

        while !batches.is_empty() {
            let batch = batches.remove(0);
            let mut form = Form::new();

            for asset in &batch {
                let data = {
                    let content = String::from(
                        &asset.path.as_path().display().to_string(),
                    );
                    content.into_bytes()
                };

                let file = Part::bytes(data)
                    .file_name(asset.id.clone())
                    // .file_name(asset.name.clone())
                    .mime_str(asset.content_type.as_str())?;
                form = form.part("file", file);
            }

            let response = self
                .client
                .post(format!("{NFT_STORAGE_API_URL}/upload"))
                .multipart(form)
                .send()
                .await?;
            let status = response.status();

            if !status.is_success() {
                // TODO: Better error handling here
                anyhow::bail!("Error uploading file");
            }
            if !batches.is_empty() {
                // wait to minimize the chance of getting caught by the rate limit
                sleep(Duration::from_millis(REQUEST_WAIT)).await;
            }
        }

        Ok(())
    }
}
