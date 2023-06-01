use std::collections::BTreeSet;

use dialoguer::{Input, MultiSelect, Select};
use gutenberg::{
    models::settings::{
        composability::Composability, minting::MintPolicies,
        royalties::RoyaltyPolicy, Orderbook, RequestPolicies, Settings,
    },
    schema::SchemaBuilder,
};

use super::{map_indices, number_validator, FromPrompt};
use crate::{
    consts::{FEATURE_OPTIONS, MINTING_OPTIONS, SUPPLY_OPTIONS},
    prelude::get_dialoguer_theme,
};

impl FromPrompt for Settings {
    fn from_prompt(schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let royalties = RoyaltyPolicy::from_prompt(schema)?;
        let mint_policies = MintPolicies::from_prompt(schema)?;

        let nft_features_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT features do you want the NFTs to add? (use [SPACEBAR] to select options)")
            .items(&FEATURE_OPTIONS)
            .interact()
            .unwrap();

        let features = map_indices(nft_features_indices, &FEATURE_OPTIONS);

        let composability =
            if features.contains(&String::from("Tradeable Traits")) {
                Some(Composability::from_prompt(schema)?)
            } else {
                None
            };

        // TODO: Design this part
        let requests = RequestPolicies::new(false, false, false); // TODO

        let orderbook = if features
            .contains(&String::from("Immediate Secondary Market Trading"))
        {
            Orderbook::Unprotected
        } else {
            Orderbook::Protected
        };

        let settings = Settings::new(
            Some(royalties),
            mint_policies,
            requests,
            composability,
            orderbook,
        );

        Ok(settings)
    }
}

impl FromPrompt for Composability {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let traits_num = Input::with_theme(&theme)
            .with_prompt("How many NFT traits will your collection have? (e.g. In tradeable traits, each trait is considered an NFT type, including the core avatar)")
            .validate_with(number_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let mut traits: BTreeSet<String> = BTreeSet::new();

        for i in 0..traits_num {
            let prompt = if i == 0 {
                String::from("Write the name of your core trait/type (This is the trait that glue all other traits together - e.g. Avatar): ")
            } else {
                format!("Write the name of the trait/type no. {} (Please add traits in descending rendering order - e.g. Hat trait should be the Hair trait):", i + 1)
            };

            let nft_trait = Input::with_theme(&theme)
                .with_prompt(prompt)
                .interact()
                .unwrap();

            traits.insert(nft_trait);
        }

        let core_trait = traits.first().unwrap().clone();

        Ok(Composability::new_from_tradeable_traits(traits, core_trait))
    }
}

impl FromPrompt for MintPolicies {
    fn from_prompt(_schema: &SchemaBuilder) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let supply_index = Select::with_theme(&theme)
            .with_prompt(
                "Which Supply Policy do you want your Collection to have?",
            )
            .items(&SUPPLY_OPTIONS)
            .interact()
            .unwrap();

        let supply_policy = SUPPLY_OPTIONS[supply_index];

        let limit = if supply_policy == "Limited" {
            Some(
                Input::with_theme(&theme)
                    .with_prompt("What is the supply limit of the Collection?")
                    .validate_with(number_validator)
                    .interact()
                    .unwrap()
                    .parse::<u64>()
                    .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.")
            )
        } else {
            None
        };

        let mint_options_indices = MultiSelect::with_theme(&theme)
            .with_prompt("What minting options do you want to use? (use [SPACEBAR] to select options)")
            .items(&MINTING_OPTIONS)
            .interact()
            .unwrap();

        let mint_options = map_indices(mint_options_indices, &MINTING_OPTIONS);

        let launchpad =
            mint_options.contains(&String::from("OriginByte Launchpad"));
        let airdrop = mint_options.contains(&String::from("NFT Airdrop"));

        Ok(MintPolicies::new(limit, launchpad, airdrop))
    }
}
