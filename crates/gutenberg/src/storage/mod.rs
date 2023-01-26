use serde::{Deserialize, Serialize};

pub mod aws;
pub mod nft_storage;
pub mod pinata;
pub mod uploader;

pub use aws::*;
pub use nft_storage::*;
pub use pinata::*;
pub use uploader::*;

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Undefined,
    Pinata(PinataConfig),
    // Bundlr(BundlrConfig),
    NftStorage(NftStorageConfig),
    // Shdw(ShdwConfig),
}

impl Default for Storage {
    fn default() -> Self {
        Storage::Undefined
    }
}
