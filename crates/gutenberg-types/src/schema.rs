//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::models::{collection::CollectionData, nft::NftData};
use crate::normalize_type;
use serde::{Deserialize, Serialize};

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// The named address that the module is published under
    package_name: String,
    #[serde(default)]
    collection: CollectionData,
    nft: NftData,
}

impl Schema {
    pub fn new(
        package_name: String,
        collection: CollectionData,
        nft: NftData,
    ) -> Schema {
        Schema {
            package_name,
            collection,
            nft,
        }
    }

    pub fn package_name(&self) -> String {
        // Since `Schema.package_name` can be deserialized from an untrusted
        // source it's fields must be escaped when preparing for display.
        normalize_type(&self.package_name).to_lowercase()
    }

    pub fn collection(&self) -> &CollectionData {
        &self.collection
    }

    pub fn nft(&self) -> &NftData {
        &self.nft
    }
}
