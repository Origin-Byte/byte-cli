use std::collections::BTreeSet;

use super::{
    address_validator, name_validator, positive_integer_validator,
    symbol_validator, url_validator, FromPrompt,
};
use crate::{
    consts::{DEFAULT_SENDER_MSG, MAX_SYMBOL_LENGTH, TX_SENDER_ADDRESS},
    prelude::get_dialoguer_theme,
};

use dialoguer::{Confirm, Input};
use gutenberg::{models::collection::CollectionData, schema::SchemaBuilder};

impl FromPrompt for CollectionData {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut collection = CollectionData::default();

        let name = Input::with_theme(&theme)
            .with_prompt("Please provide the name of the Collection:")
            .validate_with(name_validator)
            .interact()
            .unwrap();

        collection.set_name(name)?;

        let description = Input::with_theme(&theme)
            .with_prompt("Please provide the description of the Collection:")
            .interact()
            .unwrap();

        collection.set_description(description);

        let symbol = Input::with_theme(&theme)
            .with_prompt(format!(
                "Please provide the symbol of the Collection? (Maximum of {} letters)",
                MAX_SYMBOL_LENGTH
            ))
            .validate_with(symbol_validator)
            .interact()
            .unwrap();

        collection.set_symbol(symbol)?;

        let has_url = Confirm::with_theme(&theme)
            .with_prompt("Do you want to add a URL to your project's website?")
            .interact()
            .unwrap();

        if has_url {
            let url = Input::with_theme(&theme)
                .with_prompt("What is the URL of the project's website?")
                .validate_with(url_validator)
                .interact()
                .unwrap();

            collection.set_url(url)?;
        };

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
                let mut address = Input::with_theme(&theme)
                    .with_prompt(format!(
                        "Please input address of the creator number {}:",
                        i + 1,
                        // if i == 0 {" (Note: The first address will receive the MintCap object)"} else {""}
                    ))
                    .default(DEFAULT_SENDER_MSG.to_string())
                    .validate_with(address_validator)
                    .interact()
                    .unwrap();

                if address == DEFAULT_SENDER_MSG.to_string() {
                    address = TX_SENDER_ADDRESS.to_string();
                }

                if creators.contains(&address) {
                    println!("The address {} has already been added, please provide a different one.", address)
                } else {
                    break address;
                }
            };

            creators.insert(address);
        }

        collection.set_creators(creators.into_iter().collect())?;

        // TODO:
        // if features.contains(&String::from("tags")) {
        //     let tag_indices = MultiSelect::with_theme(&theme)
        //         .with_prompt("Which tags do you want to add? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        //         .items(&TAG_OPTIONS)
        //         .interact()
        //         .unwrap();

        //     let tags =
        //         Tags::new(&super::map_indices(tag_indices, &TAG_OPTIONS))?;

        //     settings.set_tags(tags);
        // }

        Ok(collection)
    }
}
