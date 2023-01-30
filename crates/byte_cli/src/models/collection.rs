use super::{address_validator, number_validator, sender, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Confirm, Input, MultiSelect};
use gutenberg::{
    models::{
        collection::Collection,
        marketplace::{Listings, Marketplace},
        nft,
        royalties::Royalties,
        tags::Tags,
    },
    Schema,
};

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

impl FromPrompt for Collection {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut collection = Collection::default();

        let name = Input::with_theme(&theme)
            .with_prompt("What is the name of the Collection?")
            .interact()
            .unwrap();

        collection.set_name(name);

        let description = Input::with_theme(&theme)
            .with_prompt("What is the description of the Collection?")
            .interact()
            .unwrap();

        collection.set_description(description);

        let symbol = Input::with_theme(&theme)
            .with_prompt("What is the symbol of the Collection?")
            .interact()
            .unwrap();

        collection.set_symbol(symbol);

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

            let tags = Tags::new(&super::map_indices(tag_indices, &TAG_OPTIONS))?;
            collection.set_tags(tags);
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

            collection.set_url(url);
        };

        Ok(collection)
    }
}
