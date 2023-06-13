use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::task::{JoinHandle, JoinSet};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use rust_sdk::metadata::{GlobalMetadata, Metadata};

/// Maximum number of concurrent tasks (this is important for tasks that handle files
/// and network connections).
pub const PARALLEL_LIMIT: usize = 45;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Item {
    pub hash: String,
    pub link: String,
}

#[derive(Debug, Default)]
pub struct StorageState {
    pub batch_pointer: u64,
    pub uploaded_items: StorageItems,
    pub missed_items: IndexMap<String, u64>,
}

#[derive(Debug, Default)]
pub struct StorageItems(pub IndexMap<String, Item>);

pub async fn upload_data(
    assets: &mut Vec<Asset>,
    metadata: Arc<GlobalMetadata>,
    uploader: &dyn Uploader,
) -> Result<()> {
    uploader.upload(assets, metadata).await?;

    Ok(())
}

// pub async fn write_state(state_path: PathBuf, url: String) -> Result<()> {
//     println!("Writing state in {:?}", state_path);
//     let mut state = try_read_state(&state_path)?;
//     println!("The state is {:?}", state);

//     state.url = Some(url);

//     write_state_file(&state, state_path.as_path()).await?;

//     Ok(())
// }

pub fn try_read_state(path_buf: &PathBuf) -> Result<Metadata> {
    let f = File::open(path_buf);

    let data = match f {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(data) => Ok(data),
            Err(err) => Err(anyhow!("The following error has occurred while reading an NFT metadata object: {},", err)),
        },
        Err(err) => Err(anyhow!("The following error has occurred while reading an NFT metadata object: {},", err)),
    }?;

    Ok(data)
}

pub async fn write_state_file(
    state: &Metadata,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    state.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}

#[derive(Debug)]
pub struct Asset {
    /// Id of the asset
    pub index: u32,
    /// Id of the asset
    pub name: String,
    /// File path of the asset
    pub path: PathBuf,
    /// MIME content type.
    pub content_type: String,
}

impl Asset {
    pub fn new(
        index: u32,
        name: String,
        path: PathBuf,
        content_type: String,
    ) -> Self {
        Asset {
            index,
            name,
            path,
            content_type,
        }
    }
}

pub struct UploadedAsset {
    /// Id of the asset
    pub index: u32,
    /// Link of the asset
    pub link: String,
}

impl UploadedAsset {
    pub fn new(index: u32, link: String) -> Self {
        UploadedAsset { index, link }
    }
}

#[async_trait]
pub trait Prepare {
    async fn prepare(&self, assets: &[Asset]) -> Result<()>;
}

#[async_trait]
pub trait Uploader: Prepare {
    async fn upload(
        &self,
        assets: &mut Vec<Asset>,
        nft_data: Arc<GlobalMetadata>,
    ) -> Result<()>;
}

#[async_trait]
pub trait ParallelUploader: Uploader + Send + Sync {
    fn upload_asset<'a>(
        &self,
        asset: Asset,
        // nft_data: RefMut<'a, u32, Metadata, RandomState>,
        metadata: Arc<GlobalMetadata>,
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
        metadata: Arc<GlobalMetadata>,
    ) -> Result<()> {
        let mut set = JoinSet::new();

        for asset in assets.drain(..) {
            set.spawn(self.upload_asset(asset, metadata.clone()));
        }

        while let Some(res) = set.join_next().await {
            res.unwrap().unwrap().unwrap();
        }

        Ok(())
    }
}
