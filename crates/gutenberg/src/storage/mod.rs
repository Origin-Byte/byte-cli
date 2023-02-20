use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub mod aws;
pub mod nft_storage;
pub mod pinata;
pub mod uploader;

pub use aws::*;
pub use nft_storage::*;
pub use pinata::*;
pub use uploader::*;

use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Pinata(PinataConfig),
    NftStorage(NftStorageConfig),
    // Bundlr(BundlrConfig),
    // Shdw(ShdwConfig),
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct Item {
    #[serde(default = "String::default")]
    pub hash: String,
    pub link: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StorageState {
    pub batch_pointer: u64,
    pub uploaded_items: StorageItems,
    pub missed_items: IndexMap<String, u64>,
}

// impl StorageState {
//     pub fn sync_state(&mut self, )
// }

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct StorageItems(pub IndexMap<String, Item>);

pub async fn upload_data(
    assets: &mut Vec<Asset>,
    state: &mut StorageState,
    lazy: bool,
    uploader: &dyn Uploader,
) -> Result<()> {
    uploader.upload(assets, state, lazy).await?;

    Ok(())
}
