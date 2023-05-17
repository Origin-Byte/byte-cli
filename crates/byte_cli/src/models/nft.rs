use super::FromPrompt;
use crate::prelude::get_dialoguer_theme;

use dialoguer::Input;
use gutenberg::{models::nft::NftData, schema::SchemaBuilder};

impl FromPrompt for NftData {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let type_name = Input::with_theme(&theme)
            .with_prompt("What name should the NFT type have?")
            .interact()
            .unwrap();

        let nft = NftData { type_name };

        Ok(nft)
    }
}
