use super::{map_indices, FromPrompt};
use crate::{
    consts::{FIELD_OPTIONS, FIELD_OPTIONS_},
    prelude::get_dialoguer_theme,
};

use dialoguer::MultiSelect;
use gutenberg::{models::nft::NftData, Schema};

impl FromPrompt for NftData {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        // TODO: The fields attribute is incompatible with tradeable traits.. or its undefined..
        let nft_field_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT fields should the NFT have? (use [SPACEBAR] to select options)")
            .items(&FIELD_OPTIONS)
            .interact()
            .unwrap();

        let mut nft_fields = map_indices(nft_field_indices, &FIELD_OPTIONS_);

        if let Some(_tags) = &schema.settings.tags {
            nft_fields.push(String::from("tags"));
        }

        let nft = NftData {
            type_name: String::from("nft"),
        };

        Ok(Some(nft))
    }
}
