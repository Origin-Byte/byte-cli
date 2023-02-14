use tokio::task::JoinHandle;

use anyhow::Result;
use async_trait::async_trait;

pub struct Asset {
    /// Id of the asset
    pub id: String,
    /// Name of the asset
    pub name: String,
    /// File path of the asset
    pub path: String,
    /// MIME content type.
    pub content_type: String,
}

#[async_trait]
pub trait Prepare {
    async fn prepare(&self, assets: &Vec<Asset>) -> Result<()>;
}

#[async_trait]
pub trait Uploader {
    async fn upload(&self, assets: &Vec<Asset>) -> Result<()>;
}

#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    fn parallel_upload(&self, asset: Asset) -> JoinHandle<Result<()>>;
}

/// Default implementation of the trait ['Uploader'](Uploader) for all ['ParallelUploader'](ParallelUploader).
#[async_trait]
impl<T: ParallelUploader> Uploader for T {
    /// Uploads assets in parallel. It creates [`self::parallel_limit()`] tasks at a time to avoid
    /// reaching the limit of concurrent files open and it syncs the cache file at every `self.parallel_limit() / 2`
    /// step.
    async fn upload(&self, _assets: &Vec<Asset>) -> Result<()> {
        todo!()
    }
}
