use super::{map_indices, FromPrompt};
use crate::cli::get_dialoguer_theme;
use dialoguer::{Input, MultiSelect};
use gutenberg_types::models::nft::{
    Burn, Dynamic, FieldType, MintCap, MintPolicies, NftData, Orderbook,
    RequestPolicies,
};

// Predefined options for various settings related to minting and burning of NFTs.
const MINTING_OPTIONS: [&str; 2] = ["OriginByte Launchpad", "NFT Airdrop"];
const BURN_PERMISSIONS: [&str; 2] = ["Permissioned", "Permissionless"];
const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];

/// Implementation of the `FromPrompt` trait for the `Burn` struct.
/// This allows creating a `Burn` instance by prompting the user for input.
impl FromPrompt for Burn {
    type Param<'a> = ();

    /// Creates a `Burn` instance from user input.
    /// Prompts the user to select the permission level for burning NFTs.
    fn from_prompt(_param: ()) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let burn_permission_idx = dialoguer::Select::with_theme(&theme)
            .with_prompt("Select the permission level for burning NFTs. (select the option you want and hit [ENTER] when done)")
            .items(&BURN_PERMISSIONS)
            .interact()
            .unwrap();

        match BURN_PERMISSIONS[burn_permission_idx] {
            "Permissioned" => Ok(Burn::Permissioned),
            "Permissionless" => Ok(Burn::Permissionless),
            // SAFETY: Prompt items will return an index within the bounds of BURN_PERMISSIONS
            _ => unreachable!(),
        }
    }
}

/// Implementation of the `FromPrompt` trait for the `Dynamic` struct.
/// Enables the creation of a `Dynamic` instance based on user input.
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

/// Implementation of the `FromPrompt` trait for the `MintCap` struct.
/// Facilitates the creation of a `MintCap` instance via user interaction.
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

        let validator =
            |input: &String| super::positive_integer_validator(input, 10_000);

        let mint_cap = match SUPPLY_OPTIONS[supply_index] {
            "Limited" => MintCap::limited(
                Input::with_theme(&theme)
                    .with_prompt("What is the supply limit of the Collection?")
                    .validate_with(validator)
                    .interact()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            ),
            _ => MintCap::unlimited(),
        };

        Ok(mint_cap)
    }
}

/// Implementing `FromPrompt` for `MintPolicies`.
/// Enables the creation of `MintPolicies` from user-selected options.
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

        let nft_data = NftData::new(
            type_name,
            Some(Burn::from_prompt(())?),
            Dynamic::from_prompt(())?,
            MintCap::from_prompt(())?,
            MintPolicies::from_prompt(())?,
            RequestPolicies::default(),
            Some(Orderbook::Protected),
            vec![
                ("name", FieldType::String),
                ("description", FieldType::String),
                ("url", FieldType::Url),
                ("attributes", FieldType::Attributes),
            ]
            .into(),
        );

        Ok(nft_data)
    }
}
