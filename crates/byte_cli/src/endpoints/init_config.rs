use crate::models::FromPrompt;
use crate::prelude::*;
use anyhow::anyhow;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use gutenberg::models::marketplace::{Listings, Marketplace};
use gutenberg::models::nft;
use gutenberg::models::royalties::Royalties;
use gutenberg::models::shared::Tags;
use gutenberg::Schema;
use serde::Serialize;
use std::fs::File;
use std::path::Path;

const TAG_OPTIONS: [&str; 11] = [
    "Art",
    "ProfilePicture",
    "Collectible",
    "GameAsset",
    "TokenisedAsset",
    "Ticker",
    "DomainName",
    "Music",
    "Video",
    "Ticket",
    "License",
];

const FIELD_OPTIONS: [&str; 3] = ["display", "url", "attributes"];
const BEHAVIOUR_OPTIONS: [&str; 2] = ["composable", "loose"];
const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];
const MINTING_OPTIONS: [&str; 3] = ["Launchpad", "Direct", "Airdrop"];

fn map_indices(indices: Vec<usize>, arr: &[&str]) -> Vec<String> {
    let vec: Vec<String> = indices
        .iter()
        .map(|index| arr[*index].to_string())
        .collect();
    vec
}

pub fn init_collection_config() -> Result<Schema, anyhow::Error> {
    let mut schema = Schema::new();
    let theme = get_dialoguer_theme();

    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number.", input))
        } else {
            Ok(())
        }
    };

    let name = Input::with_theme(&theme)
        .with_prompt("What is the name of the Collection?")
        .interact()
        .unwrap();

    schema.collection.set_name(name);

    let description = Input::with_theme(&theme)
        .with_prompt("What is the description of the Collection?")
        .interact()
        .unwrap();

    schema.collection.set_description(description);

    let symbol = Input::with_theme(&theme)
        .with_prompt("What is the symbol of the Collection?")
        .interact()
        .unwrap();

    schema.collection.set_symbol(symbol);

    let has_tags = Confirm::with_theme(&theme)
        .with_prompt("Do you want to add Tags to your Collection?")
        .interact()
        .unwrap();

    if has_tags {
        let tag_indices = MultiSelect::with_theme(&theme)
        .with_prompt("Which tags do you want to add? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&TAG_OPTIONS)
        .interact()
        .unwrap();

        let tags = Tags::new(&map_indices(tag_indices, &TAG_OPTIONS))?;
        schema.collection.set_tags(tags);
    }

    let has_url = Confirm::with_theme(&theme)
        .with_prompt("Do you want to add a URL to your Collection Website?")
        .interact()
        .unwrap();

    if has_url {
        let url = Input::with_theme(&theme)
            .with_prompt("What is the URL of the Collection Website?")
            .interact()
            .unwrap();

        schema.collection.set_url(url);
    };

    let nft_field_indices = MultiSelect::with_theme(&theme)
        .with_prompt("Which NFT fields do you want the NFTs to have? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&FIELD_OPTIONS)
        .interact()
        .unwrap();

    let mut nft_fields = map_indices(nft_field_indices, &FIELD_OPTIONS);

    // Since the creator has already mentioned that the Collection has Tags
    if has_tags {
        nft_fields.push("tags".to_string());
    };

    schema.nft.fields = nft::Fields::new_from(nft_fields).unwrap();

    let nft_behaviour_indices = MultiSelect::with_theme(&theme)
        .with_prompt("Which NFT behaviours do you want the NFTs to have? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&BEHAVIOUR_OPTIONS)
        .interact()
        .unwrap();

    let nft_behaviours = map_indices(nft_behaviour_indices, &BEHAVIOUR_OPTIONS);

    schema.nft.behaviours = nft::Behaviours::new_from(nft_behaviours).unwrap();

    let supply_index = Select::with_theme(&theme)
        .with_prompt("Which Supply Policy do you want your Collection to have?")
        .items(&SUPPLY_OPTIONS)
        .interact()
        .unwrap();

    let supply_policy = SUPPLY_OPTIONS[supply_index];

    let mut limit = Option::None;

    if supply_policy == "Limited" {
        limit = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the supply limit of the Collection?")
                .validate_with(number_validator)
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.")
        );
    }

    schema.nft.supply_policy =
        nft::SupplyPolicy::new_from(supply_policy, limit).unwrap();

    schema.royalties = Royalties::from_prompt()?;

    let mint_strategy_indices = MultiSelect::with_theme(&theme)
        .with_prompt("Which minting strategies do you plan using? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&MINTING_OPTIONS)
        .interact()
        .unwrap();

    let mint_strategies = map_indices(mint_strategy_indices, &MINTING_OPTIONS);

    let contains_launchpad = mint_strategies.contains(&"Launchpad".to_owned());

    schema.nft.mint_strategy =
        nft::MintStrategy::new_from(mint_strategies).unwrap();

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

pub fn write_config(
    schema: &Schema,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let ser = &mut serde_json::Serializer::new(file);
    schema.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}
