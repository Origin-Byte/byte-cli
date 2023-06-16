use super::{map_indices, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Input, MultiSelect};
#[cfg(feature = "full")]
use gutenberg::models::nft::{Burn, Dynamic, MintCap, Orderbook};
use gutenberg::models::nft::{MintPolicies, NftData, RequestPolicies};

const MINTING_OPTIONS: [&str; 2] = ["OriginByte Launchpad", "NFT Airdrop"];
#[cfg(feature = "full")]
const BURN_PERMISSIONS: [&str; 3] = ["None", "Permissioned", "Permissionless"];
#[cfg(feature = "full")]
const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];

#[cfg(feature = "full")]
impl FromPrompt for Burn {
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        use std::str::FromStr;

        let theme = get_dialoguer_theme();

        let burn_permission_idx = dialoguer::Select::with_theme(&theme)
            .with_prompt("Select the permission level for burning NFTs. (select the option you want and hit [ENTER] when done)")
            .items(&BURN_PERMISSIONS)
            .interact()
            .unwrap();

        Ok(Burn::from_str(BURN_PERMISSIONS[burn_permission_idx]).unwrap())
    }
}

#[cfg(feature = "full")]
impl FromPrompt for Dynamic {
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let dynamic = dialoguer::Confirm::with_theme(&theme)
            .with_prompt("Is your NFT dynamic?")
            .interact()
            .unwrap();

        Ok(Dynamic::new(dynamic))
    }
}

#[cfg(feature = "full")]
impl FromPrompt for MintCap {
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let supply_index = dialoguer::Select::with_theme(&theme)
            .with_prompt(
                "Which supply policy do you want your Collection to have?",
            )
            .items(&SUPPLY_OPTIONS)
            .interact()
            .unwrap();

        let limit = match SUPPLY_OPTIONS[supply_index] {
            "Limited" => Some(
                Input::with_theme(&theme)
                    .with_prompt("What is the supply limit of the Collection?")
                    .validate_with(super::positive_integer_validator)
                    .interact()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            ),
            _ => None,
        };

        Ok(MintCap::new(limit))
    }
}

impl FromPrompt for MintPolicies {
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
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
    type Param<'a> = ();

    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let type_name = Input::with_theme(&theme)
            .with_prompt("What name should the NFT type have?")
            .interact()
            .unwrap();

        #[cfg(feature = "full")]
        let nft_data = NftData::new(
            type_name,
            Burn::from_prompt(())?,
            Dynamic::from_prompt(())?,
            MintCap::from_prompt(())?,
            MintPolicies::from_prompt(())?,
            RequestPolicies::default(),
            Orderbook::Protected,
        );
        #[cfg(not(feature = "full"))]
        let nft_data = NftData::new(
            type_name,
            MintPolicies::from_prompt(())?,
            RequestPolicies::default(),
        );

        Ok(nft_data)
    }
}
