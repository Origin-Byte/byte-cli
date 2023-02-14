use crate::models::FromPrompt;

use gutenberg::{
    models::{collection::CollectionData, nft::NftData, settings::Settings},
    Schema,
};

pub fn init_collection_config(
    mut schema: Schema,
    to_complete: bool,
) -> Result<Schema, anyhow::Error> {
    if !to_complete || schema.collection.is_empty() {
        schema.collection = CollectionData::from_prompt(&schema)?.unwrap();
    }

    if !to_complete || schema.nft.is_empty() {
        schema.nft = NftData::from_prompt(&schema)?.unwrap();
    }

    if !to_complete || schema.settings.is_empty() {
        schema.settings = Settings::from_prompt(&schema)?.unwrap();
    }

    Ok(schema)
}
