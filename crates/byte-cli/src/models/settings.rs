use super::{map_indices, number_validator, FromPrompt};
use crate::{consts::FEATURE_OPTIONS, prelude::get_dialoguer_theme};
use dialoguer::{Input, MultiSelect, Select};
use gutenberg::{
    models::collection::{Orderbook, RequestPolicies, RoyaltyPolicy},
    models::nft::MintPolicies,
    schema::SchemaBuilder,
};
use std::collections::BTreeSet;

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
