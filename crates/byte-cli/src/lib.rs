pub mod cli;
pub mod consts;
pub mod err;
pub mod io;
pub mod models;

use gutenberg_types::models::{collection::CollectionData, nft::NftData};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default)]
pub struct SchemaBuilder {
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub collection: Option<CollectionData>,
    #[serde(default)]
    pub nft: Option<NftData>,
}
