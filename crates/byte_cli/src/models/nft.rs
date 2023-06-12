use std::str::FromStr;

use super::{map_indices, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Confirm, Input, MultiSelect, Select};
use gutenberg::{
    models::nft::{Burn, MintPolicies, NftData},
    schema::SchemaBuilder,
};

const MINTING_OPTIONS: [&str; 2] = ["OriginByte Launchpad", "NFT Airdrop"];
const BURN_PERMISSIONS: [&str; 3] = ["None", "Permissioned", "Permissionless"];

impl FromPrompt for MintPolicies {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mint_options_indices = MultiSelect::with_theme(&theme)
            .with_prompt("What minting options do you want to use? (use [SPACEBAR] to select options)")
            .items(&MINTING_OPTIONS)
            .interact()
            .unwrap();

        let mint_options = map_indices(mint_options_indices, &MINTING_OPTIONS);

        let launchpad = mint_options.contains(&"OriginByte Launchpad");
        let airdrop = mint_options.contains(&"NFT Airdrop");

        Ok(MintPolicies::new(launchpad, airdrop))
    }
}

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
            dynamic,
            MintPolicies::new(true, true),
        );

        Ok(nft)
    }
}
