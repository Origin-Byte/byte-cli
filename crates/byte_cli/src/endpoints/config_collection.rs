use crate::models::FromPrompt;

use console::style;
use gutenberg::{
    models::{collection::CollectionData, nft::NftData, settings::Settings},
    schema::SchemaBuilder,
};

pub fn init_collection_config(
    mut builder: SchemaBuilder,
    to_complete: bool,
) -> Result<SchemaBuilder, anyhow::Error> {
    println!("{}",
        style("Welcome to Byte CLI! We're ready to begin setting up your NFT collection.").blue().bold().dim()
    );

    if !to_complete || builder.collection.is_none() {
        println!(
            "{}",
            style("To begin, let's configure some collection level metadata.")
                .blue()
                .bold()
                .dim()
        );

        builder.collection = Some(CollectionData::from_prompt(&builder)?);
    }

    if !to_complete {
        println!(
            "{}",
            style("Let us now configure some NFT level metadata.")
                .blue()
                .bold()
                .dim()
        );

        builder.nft = Some(NftData::from_prompt(&builder)?);
    }

    println!(
        "{}",
        style("Awesome! As a last step we need to configure some settings.")
            .blue()
            .bold()
            .dim()
    );

    if !to_complete || builder.settings.is_none() {
        builder.settings = Some(Settings::from_prompt(&builder)?);
    }

    println!(
        "{}",
        style("Congrats! The collection has been configured.")
            .blue()
            .bold()
            .dim()
    );

    Ok(builder)
}
