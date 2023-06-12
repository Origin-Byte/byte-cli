use std::collections::BTreeSet;

use super::{
    address_validator, name_validator, positive_integer_validator,
    symbol_validator, url_validator, FromPrompt,
};
use crate::{consts::MAX_SYMBOL_LENGTH, prelude::get_dialoguer_theme};

use dialoguer::{Confirm, Input, Select};
use gutenberg::{
    models::{
        collection::{
            CollectionData, MintCap, Orderbook, RequestPolicies, RoyaltyPolicy,
            Supply,
        },
        Address,
    },
    schema::SchemaBuilder,
};

const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];

impl FromPrompt for MintCap {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let supply_index = Select::with_theme(&theme)
            .with_prompt(
                "Which supply policy do you want your Collection to have?",
            )
            .items(&SUPPLY_OPTIONS)
            .interact()
            .unwrap();

        let limit = match SUPPLY_OPTIONS[supply_index] {
            "Limited" => Some(
                Input::with_theme(&theme)
                    .with_prompt("What is the supply limit of the Collection?")
                    .validate_with(positive_integer_validator)
                    .interact()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            ),
            _ => None,
        };

        Ok(MintCap::new(limit))
    }
}

impl FromPrompt for CollectionData {
    fn from_prompt(schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let name = Input::with_theme(&theme)
            .with_prompt("Please provide the name of the Collection:")
            .validate_with(name_validator)
            .interact()
            .unwrap();

        let description = Input::with_theme(&theme)
            .with_prompt("Please provide the description of the Collection:")
            .interact()
            .unwrap();

        let symbol = Input::with_theme(&theme)
            .with_prompt(format!(
                "Please provide the symbol of the Collection? (Maximum of {} letters)",
                MAX_SYMBOL_LENGTH
            ))
            .validate_with(symbol_validator)
            .interact()
            .unwrap();

        let has_url = Confirm::with_theme(&theme)
            .with_prompt("Do you want to add a URL to your project's website?")
            .interact()
            .unwrap();

        let url = has_url.then(|| {
            Input::with_theme(&theme)
                .with_prompt("What is the URL of the project's website?")
                .validate_with(url_validator)
                .interact()
                .unwrap()
        });

        // TODO: Separate into `Creators::from_prompt`
        let creators_num = Input::with_theme(&theme)
            .with_prompt("How many creator addresses are there?")
            .validate_with(positive_integer_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let mut creators = BTreeSet::new();

        for i in 0..creators_num {
            // Loop checks if address is not duplicated
            let address = loop {
                let address = Input::with_theme(&theme)
                    .with_prompt(format!(
                        "Please input address of the creator number {}:",
                        i + 1,
                        // if i == 0 {" (Note: The first address will receive the MintCap object)"} else {""}
                    ))
                    .validate_with(address_validator)
                    .interact()
                    .map(Address::new)
                    .unwrap()?;

                if creators.contains(&address) {
                    println!("The address {} has already been added, please provide a different one.", address)
                } else {
                    break address;
                }
            };

            creators.insert(address);
        }

        Ok(CollectionData::new(
            name.to_lowercase(),
            Some(description),
            Some(symbol.to_uppercase()),
            url,
            creators.into_iter().collect(),
            // Use tracked supply as default as it is most compatible
            Supply::tracked(),
            MintCap::from_prompt(schema)?,
            Some(RoyaltyPolicy::from_prompt(schema)?),
            // TODO: Tags
            None,
            RequestPolicies::default(),
            Orderbook::Protected,
        ))
    }
}
