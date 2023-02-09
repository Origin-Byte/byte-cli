use crate::models::FromPrompt;
use crate::prelude::*;

use dialoguer::Confirm;
use gutenberg::{
    models::{collection::CollectionData, nft::NftData, settings::Settings},
    Schema,
};

pub fn init_collection_config(
    mut schema: Schema,
) -> Result<Schema, anyhow::Error> {
    schema.collection = CollectionData::from_prompt(&schema)?.unwrap();

    schema.nft = NftData::from_prompt(&schema)?.unwrap();

    schema.settings = Settings::from_prompt(&schema)?.unwrap();

    Ok(schema)
}
