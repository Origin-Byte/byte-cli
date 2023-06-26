use crate::storage::{
    aws::AWSConfig,
    pinata::PinataConfig, // nft_storage::NftStorageConfig
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Pinata(PinataConfig),
    // NftStorage(NftStorageConfig),
}
