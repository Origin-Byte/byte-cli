use crate::models::FromPrompt;

use console::style;
use gutenberg::{
    models::launchpad::{marketplace::Marketplace, Launchpad},
    Schema,
};

pub fn init_marketplace_config(
    mut schema: Schema,
) -> Result<Schema, anyhow::Error> {
    println!(
        "{}",
        style(
            "Hey there! Let's setup your OriginByte-powered Marketplace on-chain."
        )
        .blue()
        .bold()
        .dim()
    );

    if schema.launchpad.is_none() {
        schema.launchpad = Some(Launchpad::default());
    }

    schema.launchpad.as_mut().unwrap().marketplace =
        Marketplace::from_prompt(&schema)?;

    Ok(schema)
}
