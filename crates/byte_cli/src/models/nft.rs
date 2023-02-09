use super::{map_indices, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::MultiSelect;
use gutenberg::{models::nft::NftData, Schema};

const FIELD_OPTIONS: [&str; 3] = ["display", "url", "attributes"];

impl FromPrompt for NftData {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let nft_field_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT fields do you want the NFTs to have? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
            .items(&FIELD_OPTIONS)
            .interact()
            .unwrap();

        let mut nft_fields = map_indices(nft_field_indices, &FIELD_OPTIONS);

        if let Some(_tags) = &schema.settings.tags {
            nft_fields.push(String::from("tags"));
        }

        let nft = NftData::new(nft_fields).unwrap();

        Ok(Some(nft))
    }
}
