use std::collections::BTreeSet;

use super::{address_validator, positive_integer_validator, FromPrompt};
use crate::{consts::TX_SENDER_ADDRESS, prelude::get_dialoguer_theme};

use dialoguer::{Confirm, Input};
use gutenberg::{models::collection::CollectionData, Schema};

impl FromPrompt for CollectionData {
    fn from_prompt(_schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut collection = CollectionData::default();

        let name = Input::with_theme(&theme)
            .with_prompt("What is the name of the Collection?")
            .interact()
            .unwrap();

        collection.set_name(name)?;

        let description = Input::with_theme(&theme)
            .with_prompt("What is the description of the Collection?")
            .interact()
            .unwrap();

        collection.set_description(description);

        let symbol = Input::with_theme(&theme)
            .with_prompt(
                "What is the symbol of the Collection? (Maximum of 5 letters)",
            )
            .interact()
            .unwrap();

        collection.set_symbol(symbol)?;

        let has_url = Confirm::with_theme(&theme)
            .with_prompt("Do you want to add a URL to your Collection Website?")
            .interact()
            .unwrap();

        if has_url {
            let url = Input::with_theme(&theme)
                .with_prompt("What is the URL of the Collection Website?")
                .interact()
                .unwrap();

            collection.set_url(url)?;
        };

        let _are_you_creator = Confirm::with_theme(&theme)
            .with_prompt("Are you the creator?")
            .interact()
            .unwrap();

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
                    .default(TX_SENDER_ADDRESS.to_string())
                    .validate_with(address_validator)
                    .interact()
                    .unwrap();

                if creators.contains(&address) {
                    println!("The address {} has already been added, please provide a different one.", address)
                } else {
                    break address;
                }
            };

            creators.insert(address);
        }

        collection.set_creators(creators)?;

        Ok(Some(collection))
    }
}

#[cfg(test)]
mod tests {
    use dialoguer::Select;

    use super::*;

    #[test]
    fn prompt_name() {
        let name = "Suimarines";

        let result: String = Input::new()
            .with_post_completion_text("Suimarines")
            .interact()
            .unwrap();

        assert_eq!(result, name);
    }
}
