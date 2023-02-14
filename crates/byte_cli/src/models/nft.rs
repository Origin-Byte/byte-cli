use super::{get_options, map_indices, number_validator, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Confirm, Input, MultiSelect, Select};
use gutenberg::{
    models::nft::{Behaviours, Fields, MintStrategy, Nft, SupplyPolicy},
};

const FIELD_OPTIONS: [&str; 3] = ["display", "url", "attributes"];
const BEHAVIOUR_OPTIONS: [&str; 2] = ["composable", "loose"];
const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];
const MINTING_OPTIONS: [&str; 3] = ["Launchpad", "Direct", "Airdrop"];

impl FromPrompt for Nft {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut nft = Nft::default();

        let nft_field_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT fields do you want the NFTs to have? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
            .items(&FIELD_OPTIONS)
            .interact()
            .unwrap();

        let mut nft_fields = map_indices(nft_field_indices, &FIELD_OPTIONS);

        nft.fields = Fields::new(nft_fields).unwrap();

        let nft_behaviour_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT behaviours do you want the NFTs to have? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
            .items(&BEHAVIOUR_OPTIONS)
            .interact()
            .unwrap();

        let nft_behaviours =
            map_indices(nft_behaviour_indices, &BEHAVIOUR_OPTIONS);

        nft.behaviours = Behaviours::new(nft_behaviours).unwrap();

        let supply_index = Select::with_theme(&theme)
            .with_prompt(
                "Which Supply Policy do you want your Collection to have?",
            )
            .items(&SUPPLY_OPTIONS)
            .interact()
            .unwrap();

        let supply_policy = SUPPLY_OPTIONS[supply_index];

        let mut limit = Option::None;

        if supply_policy == "Limited" {
            limit = Some(
            Input::with_theme(&theme)
                .with_prompt("What is the supply limit of the Collection?")
                .validate_with(number_validator)
                .interact()
                .unwrap()
                .parse::<u64>()
                .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.")
        );
        }

        nft.supply_policy = SupplyPolicy::new(supply_policy, limit).unwrap();

        let mint_strategies = get_options(
            &theme,
            "Which minting strategies do you plan on using? Choose at least one (use [SPACEBAR] to select options you want and hit [ENTER] when done)",
            &MINTING_OPTIONS
        )?;

        nft.mint_strategy = MintStrategy::new(mint_strategies).unwrap();

        Ok(nft)
    }
}
