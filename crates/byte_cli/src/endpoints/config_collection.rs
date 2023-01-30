use std::path::PathBuf;

use crate::io;
use crate::models::FromPrompt;
use crate::prelude::*;

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use gutenberg::{
    models::{
        collection::Collection,
        marketplace::{Listings, Marketplace},
        nft::Nft,
        royalties::Royalties,
        tags::Tags,
    },
    Schema,
};

pub fn init_collection_config(
    mut schema: Schema,
) -> Result<Schema, anyhow::Error> {
    let theme = get_dialoguer_theme();

    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number.", input))
        } else {
            Ok(())
        }
    };

    schema.collection = Collection::from_prompt()?;

    schema.nft = Nft::from_prompt()?;

    // Since the creator has already mentioned that the Collection has Tags
    if schema.collection.tags.has_tags() {
        schema.nft.fields.tags = true;
    };

    schema.royalties = Royalties::from_prompt()?;

    let contains_launchpad = schema.nft.mint_strategy.launchpad;

    if contains_launchpad {
        let marketplace = Marketplace::from_prompt()?;
        let mut listings = Listings::from_prompt()?;
        for listing in listings.0.iter_mut() {
            listing.admin = marketplace.admin.clone();
            listing.receiver = marketplace.receiver.clone();
        }

        schema.listings = Some(listings);
    }

    Ok(schema)
}
