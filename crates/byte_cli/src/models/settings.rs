use std::collections::BTreeSet;

use dialoguer::{Confirm, Input, MultiSelect, Select};
use gutenberg::{
    models::{
        marketplace::Listings,
        royalties::RoyaltyPolicy,
        settings::{Composability, MintPolicies, Settings},
        supply_policy::SupplyPolicy,
        tags::Tags,
    },
    Schema,
};

use super::{get_options, map_indices, number_validator, FromPrompt};
use crate::{
    consts::{FEATURE_OPTIONS, MINTING_OPTIONS, SUPPLY_OPTIONS, TAG_OPTIONS},
    prelude::get_dialoguer_theme,
};

impl FromPrompt for Settings {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut settings = Settings::default();

        let royalties = RoyaltyPolicy::from_prompt(schema)?;

        if let Some(royalties) = royalties {
            settings.set_royalties(royalties);
        }

        let mint_policies = get_options(
            &theme,
            "Which minting policies do you plan on using? Choose at least one (use [SPACEBAR] to select options you want and hit [ENTER] when done)",
            &MINTING_OPTIONS
        )?;

        settings.set_mint_policies(MintPolicies::new(mint_policies)?);

        let nft_features_indices = MultiSelect::with_theme(&theme)
            .with_prompt("Which NFT features do you want the NFTs to add? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
            .items(&FEATURE_OPTIONS)
            .interact()
            .unwrap();

        let features = map_indices(nft_features_indices, &FEATURE_OPTIONS);

        if features.contains(&String::from("tags")) {
            let tag_indices = MultiSelect::with_theme(&theme)
                .with_prompt("Which tags do you want to add? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
                .items(&TAG_OPTIONS)
                .interact()
                .unwrap();

            let tags =
                Tags::new(&super::map_indices(tag_indices, &TAG_OPTIONS))?;

            settings.set_tags(tags);
        }

        if features.contains(&String::from("tradeable_traits")) {
            let composability = Composability::from_prompt(&schema)?.unwrap();
            settings.set_composability(composability);
        }

        if features.contains(&String::from("loose")) {
            settings.set_loose(true);
        }

        if settings.mint_policies.launchpad {
            let listings = Listings::from_prompt(&schema)?.unwrap();

            settings.listings = Some(listings);
        }

        Ok(Some(settings))
    }
}

impl FromPrompt for Composability {
    fn from_prompt(_schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let traits_num = Input::with_theme(&theme)
            .with_prompt("How many NFT traits/types will your collection have? (e.g. In tradeable traits, each trait is considered an NFT type, including the core avatar)")
            .validate_with(number_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let mut traits: BTreeSet<String> = BTreeSet::new();

        for i in 0..traits_num {
            let prompt = if i == 0 {
                format!(
                    "Write the name of your core trait/type (This is the trait that glue all other traits together - e.g. Avatar): "
                )
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

        Ok(Some(Composability::new_from_tradeable_traits(
            traits, core_trait,
        )))
    }
}

impl FromPrompt for SupplyPolicy {
    fn from_prompt(_schema: &Schema) -> Result<Option<Self>, anyhow::Error>
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

        let mut limit = Option::None;
        let mut frozen = Option::None;

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

            frozen = Some(
                Confirm::with_theme(&theme)
                    .with_prompt("Do you want to freeze the supply? (You can also freeze later)")
                    .interact()
                    .unwrap()
            );
        }

        Ok(Some(SupplyPolicy::new(supply_policy, limit, frozen)?))
    }
}
