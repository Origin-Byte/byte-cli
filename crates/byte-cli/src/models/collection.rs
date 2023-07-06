use std::collections::BTreeSet;

use super::{
    address_validator, name_validator, positive_integer_validator,
    symbol_validator, url_validator, FromPrompt,
};
use crate::{cli::get_dialoguer_theme, consts::MAX_SYMBOL_LENGTH};

use dialoguer::{Confirm, Input};
use package_manager::Address;

impl FromPrompt for CollectionData {
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let name = Input::with_theme(&theme)
            .with_prompt("Collection name:")
            .validate_with(name_validator)
            .interact()
            .unwrap();

        let description = Input::with_theme(&theme)
            .with_prompt("Collection description:")
            .interact()
            .unwrap();

        let symbol = Input::with_theme(&theme)
            .with_prompt(format!(
                "Collection symbol (Maximum of {} letters):",
                MAX_SYMBOL_LENGTH
            ))
            .validate_with(symbol_validator)
            .interact()
            .unwrap();

        let has_url = Confirm::with_theme(&theme)
            .with_prompt("Do you want to add a Link to the project website?")
            .interact()
            .unwrap();

        let url = has_url.then(|| {
            Input::with_theme(&theme)
                .with_prompt("Project website link:")
                .validate_with(url_validator)
                .interact()
                .unwrap()
        });

        // TODO: Separate into `Creators::from_prompt`
        let creators_num = Input::with_theme(&theme)
            .with_prompt("Number of creator addresses:")
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
                    .map(|address| Address::new(&address))
                    .unwrap()?;

                if creators.contains(&address) {
                    println!("The address {} has already been added, please provide a different one.", address)
                } else {
                    break address;
                }
            };

            creators.insert(address);
        }

        let creators: Vec<Address> = creators.into_iter().collect();
        let royalties = RoyaltyPolicy::from_prompt(creators.as_slice())?;

        let collection_data = CollectionData::new(
            Some(name.to_lowercase()),
            Some(description),
            Some(symbol.to_uppercase()),
            url,
            creators,
            // Use tracked supply as default as it is most compatible
            Supply::tracked(),
            Some(royalties),
            // TODO: Tags
            None,
        );

        Ok(collection_data)
    }
}
