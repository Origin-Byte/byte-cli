use gutenberg::models::{collection::CollectionData, nft::NftData};
use serde::{Deserialize, Serialize};

pub mod cli;
pub mod consts;
pub mod err;
pub mod io;
pub mod models;

#[derive(Deserialize, Serialize, Default)]
pub struct SchemaBuilder {
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub collection: Option<CollectionData>,
    #[serde(default)]
    pub nft: Option<NftData>,
}
