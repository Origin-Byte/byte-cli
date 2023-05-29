use std::str::FromStr;

use super::FromPrompt;
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Confirm, Input, Select};
use gutenberg::{
    models::nft::{burn::Burn, NftData},
    schema::SchemaBuilder,
};

const BURN_PERMISSIONS: [&'static str; 3] =
    ["None", "Permissioned", "Permissionless"];

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

        let burn_permission_idx = Select::with_theme(&theme)
            .with_prompt("Select the permission level for burning NFTs. (select the option you want and hit [ENTER] when done)")
            .items(&BURN_PERMISSIONS)
            .interact()
            .unwrap();

        let dynamic = Confirm::with_theme(&theme)
            .with_prompt("Is your NFT dynamic?")
            .interact()
            .unwrap();

        let nft = NftData::new(
            type_name,
            Burn::from_str(BURN_PERMISSIONS[burn_permission_idx]).unwrap(),
            dynamic.into(),
        );

        Ok(nft)
    }
}
