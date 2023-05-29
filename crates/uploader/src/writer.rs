use crate::storage::{
    aws::AWSConfig, nft_storage::NftStorageConfig, pinata::PinataConfig,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Pinata(PinataConfig),
    NftStorage(NftStorageConfig),
    // Bundlr(BundlrConfig),
    // Shdw(ShdwConfig),
}

// TODO: Add method to upload
