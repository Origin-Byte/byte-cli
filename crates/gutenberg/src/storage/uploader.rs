use std::path::PathBuf;

use tokio::task::{JoinHandle, JoinSet};

use anyhow::Result;
use async_trait::async_trait;

use super::StorageState;

/// Maximum number of concurrent tasks (this is important for tasks that handle files
/// and network connections).
pub const PARALLEL_LIMIT: usize = 45;

#[derive(Debug)]
pub struct Asset {
    /// Id of the asset
    pub id: String,
    /// File path of the asset
    pub path: PathBuf,
    /// MIME content type.
    pub content_type: String,
}

impl Asset {
    pub fn new(id: String, path: PathBuf, content_type: String) -> Self {
        Asset {
            id,
            path,
            content_type,
        }
    }
}

pub struct UploadedAsset {
    /// Id of the asset
    pub id: String,
    /// Link of the asset
    pub link: String,
}

impl UploadedAsset {
    pub fn new(id: String, link: String) -> Self {
        UploadedAsset { id, link }
    }
}

#[async_trait]
pub trait Prepare {
    async fn prepare(&self, assets: &Vec<Asset>) -> Result<()>;
}

#[async_trait]
pub trait Uploader {
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        state: &mut StorageState,
        lazy: bool,
    ) -> Result<()>;
}

#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    fn upload_asset(
        &self,
        // set: &mut JoinSet<()>,
        // tx: Sender<UploadedAsset>,
        asset: Asset,
    ) -> JoinHandle<Result<UploadedAsset>>;
}

/// Default implementation of the trait ['Uploader'](Uploader) for all ['ParallelUploader'](ParallelUploader).
#[async_trait]
impl<T: ParallelUploader> Uploader for T {
    /// Uploads assets in parallel. It creates [`self::parallel_limit()`] tasks at a time to avoid
    /// reaching the limit of concurrent files open and it syncs the cache file at every `self.parallel_limit() / 2`
    /// step.
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        _state: &mut StorageState,
        _lazy: bool,
    ) -> Result<()> {
        println!("I am here...");
        let mut set = JoinSet::new();

        // TODO: Cache strategy - need to add fault tolerance and recovery strategy
        // TODO: Fail immediately as soon as one thread fails

        for asset in assets.drain(..) {
            set.spawn(self.upload_asset(asset));
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap().unwrap();
        }

        println!("SERVICE DONE");

        Ok(())
    }
}
