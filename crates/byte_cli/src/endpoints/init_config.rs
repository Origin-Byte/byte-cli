use crate::prelude::*;
use anyhow::Result;
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Select};
use gutenberg::{
    models::nft,
    schema,
    types::{Listing, Market, Royalties},
};
use hex;

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
const ROYALTY_OPTIONS: [&str; 3] = ["Proportional", "Constant", "None"];
const MARKET_OPTIONS: [&str; 2] = ["FixedPrice", "DutchAuction"];

pub fn get_dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new(),
        checked_item_prefix: style("✔".to_string()).green().force_styling(true),
        unchecked_item_prefix: style("✔".to_string())
            .black()
            .force_styling(true),
        ..Default::default()
    }
}

pub fn map_indices(indices: Vec<usize>, arr: &[&str]) -> Vec<String> {
    let vec: Vec<String> = indices
        .iter()
        .map(|index| arr[*index].to_string())
        .collect();
    vec
}

pub fn init_collection_config() {
    let mut schema = schema::Schema::new();
    let theme = get_dialoguer_theme();

    let string_validator = |_input: &String| -> Result<(), String> { Ok(()) };

    let address_validator = |input: &String| -> Result<(), CliError> {
        let hexa_str = if &input[..2] == "0x" {
            &input[2..]
        } else {
            &input
        };

        let hexa =
            hex::decode(hexa_str).map_err(|_| CliError::InvalidAddress)?;

        if hexa.len() != 20 {
            Err(CliError::InvalidAddress)
        } else {
            Ok(())
        }
    };

    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number.", input))
        } else {
            Ok(())
        }
    };

    let name = Input::with_theme(&theme)
        .with_prompt("What is the name of the Collection?")
        .validate_with(string_validator)
        .interact()
        .unwrap();

    schema.collection.set_name(name);

    let description = Input::with_theme(&theme)
        .with_prompt("What is the description of the Collection?")
        .validate_with(string_validator)
        .interact()
        .unwrap();

    schema.collection.set_description(description);

    let symbol = Input::with_theme(&theme)
        .with_prompt("What is the symbol of the Collection?")
        .validate_with(string_validator)
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

        let tags = map_indices(tag_indices, &TAG_OPTIONS);

        schema.collection.set_tags(&tags).unwrap();
    }

    let has_url = Confirm::with_theme(&theme)
        .with_prompt("Do you want to add a URL to your Collection Website?")
        .interact()
        .unwrap();

    if has_url {
        let url = Input::with_theme(&theme)
            .with_prompt("What is the URL of the Collection Website?")
            .validate_with(string_validator)
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

    let royalty_index = Select::with_theme(&theme)
        .with_prompt(
            "Which Royalty Policy do you want your Collection to have?",
        )
        .items(&ROYALTY_OPTIONS)
        .interact()
        .unwrap();

    let royalty_policy = ROYALTY_OPTIONS[royalty_index];

    let mut fee = Option::None;

    if royalty_policy == "Proportional" {
        fee = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the royalty fee in Basis Points?")
                .validate_with(number_validator)
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.")
        );
    }
    if royalty_policy == "Constant" {
        fee = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the constant royalty commission?")
                .validate_with(number_validator)
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.")
        );
    }

    schema.royalties = Royalties::new_from(royalty_policy, fee).unwrap();

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
        let admin_address = Input::with_theme(&theme)
            .with_prompt("What is the address of the Listing administrator?")
            .validate_with(address_validator)
            .interact()
            .unwrap();

        let receiver_address = Input::with_theme(&theme)
            .with_prompt("What is the address that receives the sale proceeds?")
            .validate_with(address_validator)
            .interact()
            .unwrap();

        let listings: u64 = Input::with_theme(&theme)
            .with_prompt(
                // TODO: The meaning of this questions may be ambiguous
                // from the perspective of the creator
                "How many Primary Market Listings do you plan on having?",
            )
            .validate_with(number_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.");

        for i in 0..listings {
            let prompt = format!(
                "What is the market primitive to use for the sale nº {}",
                i + 1
            );

            let market_index = Select::with_theme(&theme)
                .with_prompt(prompt)
                .items(&MARKET_OPTIONS)
                .interact()
                .unwrap();

            let is_whitelisted = Confirm::with_theme(&theme)
                .with_prompt("What is it a whitelisted sale?")
                .interact()
                .unwrap();

            let market: Market;

            match MARKET_OPTIONS[market_index] {
                "FixedPrice" => {
                    let price = Input::with_theme(&theme)
                        .with_prompt("What is the fixed price of the sale?")
                        .validate_with(number_validator)
                        .interact()
                        .unwrap()
                        .parse::<u64>()
                        .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.");

                    market = Market::FixedPrice {
                        token: "sui::sui::SUI".to_string(),
                        price,
                        is_whitelisted,
                    };
                }
                "DutchAuction" => {
                    let reserve_price = Input::with_theme(&theme)
                        .with_prompt(
                            "What is the reserve price of the auction?",
                        )
                        .validate_with(number_validator)
                        .interact()
                        .unwrap()
                        .parse::<u64>()
                        .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.");

                    market = Market::DutchAuction {
                        token: "sui::sui::SUI".to_string(),
                        reserve_price,
                        is_whitelisted,
                    };
                }
                _ => {
                    eprintln!("TODO: This error handling");
                    std::process::exit(2);
                }
            }

            let listing = Listing::new(
                admin_address.as_str(),
                receiver_address.as_str(),
                market,
            );

            schema.add_listing(listing);
        }
    }
}
