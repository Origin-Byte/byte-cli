use crate::models::FromPrompt;

use console::style;
use gutenberg::{
    models::{collection::CollectionData, nft::NftData, settings::Settings},
    Schema,
};

pub fn init_collection_config(
    mut schema: Schema,
    to_complete: bool,
) -> Result<Schema, anyhow::Error> {
    println!("{}",
        style("Welcome to Byte CLI! We're ready to begin setting up your NFT collection.").blue().bold().dim()
    );

    if !to_complete || schema.collection.is_empty() {
        println!(
            "{}",
            style("To begin, let's configure some collection level metadata.")
                .blue()
                .bold()
                .dim()
        );

        schema.collection = CollectionData::from_prompt(&schema)?.unwrap();
    }

    if !to_complete || schema.nft.is_empty() {
        println!(
            "{}",
            style("Let us now configure some NFT level metadata.")
                .blue()
                .bold()
                .dim()
        );

        schema.nft = NftData::from_prompt(&schema)?.unwrap();
    }

    println!(
        "{}",
        style("Awesome! As a last step we need to configure some settings.")
            .blue()
            .bold()
            .dim()
    );

    if !to_complete || schema.settings.is_empty() {
        schema.settings = Settings::from_prompt(&schema)?.unwrap();
    }

    println!(
        "{}",
        style("Congrats! The collection has been configured.")
            .blue()
            .bold()
            .dim()
    );

    Ok(schema)
}
