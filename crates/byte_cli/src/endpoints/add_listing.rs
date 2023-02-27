use crate::models::FromPrompt;

use console::style;
use gutenberg::{
    models::launchpad::{
        listing::Listing, marketplace::Marketplace, Launchpad,
    },
    Schema,
};

pub fn add_listing_config(mut schema: Schema) -> Result<Schema, anyhow::Error> {
    println!(
        "{}",
        style("Hey there! Let's setup an NFT listing.")
            .blue()
            .bold()
            .dim()
    );

    let listing = Listing::from_prompt(&schema)?.unwrap();

    if schema.launchpad.is_none() {
        schema.launchpad = Some(Launchpad::default());
    }

    if let Some(launchpad) = &mut schema.launchpad {
        launchpad.listings.0.push(listing);
    }

    println!(
        "{}",
        style("Congrats! The listing has been configured.")
            .blue()
            .bold()
            .dim()
    );

    Ok(schema)
}
