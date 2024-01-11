use super::{address_validator, bps_validator, number_validator, FromPrompt};
use crate::{cli::get_dialoguer_theme, consts::TX_SENDER_ADDRESS};
use anyhow::Result;
use dialoguer::{Confirm, Input};
use gutenberg_types::models::{
    address::Address,
    collection::{RoyaltyPolicy, Share},
};
use std::collections::BTreeSet;

/// Implementation of the `FromPrompt` trait for `RoyaltyPolicy`.
impl FromPrompt for RoyaltyPolicy {
    /// Defines the parameter type for this implementation.
    type Param<'a> = &'a [Address];

    /// Creates a `RoyaltyPolicy` instance interactively from user prompts.
    ///
    /// # Arguments
    /// * `creators` - A slice of `Address` representing the creators.
    ///
    /// # Returns
    /// * `Result<Self, anyhow::Error>` - A result containing the `RoyaltyPolicy` or an error.
    fn from_prompt(creators: &[Address]) -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let mut policy = get_policy_type()?;

        if are_royalty_owners_creators() {
            let shares = royalty_shares(creators);
            policy.add_beneficiary_vecs(creators, &shares);
        } else {
            let mut beneficiaries = get_beneficiaries()?;
            policy.add_beneficiaries(&mut beneficiaries);
        };

        Ok(policy)
    }
}

/// Fetches the type of royalty policy based on user input.
///
/// # Returns
/// * `Result<RoyaltyPolicy, anyhow::Error>` - A result containing the `RoyaltyPolicy` or an error.
pub fn get_policy_type() -> Result<RoyaltyPolicy, anyhow::Error> {
    let theme = get_dialoguer_theme();

    let bps = Input::with_theme(&theme)
        .with_prompt("What is the proportional royalty fee in basis points?")
        .validate_with(bps_validator)
        .interact()?
        .parse::<u64>()?;

    Ok(RoyaltyPolicy::Proportional {
        shares: BTreeSet::new(),
        collection_royalty_bps: bps,
    })
}

/// Determines if the royalty owners are the same as the creators based on user confirmation.
///
/// # Returns
/// * `bool` - `true` if the royalty beneficiary addresses are the same as the creator addresses, else `false`.
pub fn are_royalty_owners_creators() -> bool {
    let theme = get_dialoguer_theme();

    Confirm::with_theme(&theme)
        .with_prompt("Are the royalty beneficiary addresses the same as the creator addresses?")
        .interact()
        .unwrap()
}

/// Interactively collects details about beneficiaries and their shares for royalty distribution.
///
/// # Returns
/// * `Result<BTreeSet<Share>>` - A result containing a set of `Share` or an error.
pub fn get_beneficiaries() -> Result<BTreeSet<Share>> {
    let theme = get_dialoguer_theme();

    let beneficiary_num = Input::with_theme(&theme)
        .with_prompt("How many royalty beneficiaries are there?")
        .validate_with(number_validator)
        .interact()
        .unwrap()
        .parse::<u64>()
        .unwrap();

    let mut shares: BTreeSet<Share> = BTreeSet::new();
    let mut beneficiaries: Vec<String> = Vec::new();
    let mut shares_remaining: u64 = 10_000;

    for i in 0..beneficiary_num {
        let address = loop {
            let address = Input::with_theme(&theme)
                .with_prompt(format!(
                    "Please input address of the beneficiary number {}?",
                    i + 1
                ))
                .default(TX_SENDER_ADDRESS.to_string())
                .validate_with(address_validator)
                .interact()
                .unwrap();

            if beneficiaries.contains(&address) {
                println!("The address {} has already been added, please provide a different one.", address)
            } else {
                break address;
            }
        };

        let share = loop {
            let share = Input::with_theme(&theme)
                .with_prompt(
                    format!("What is the percentage share (in basis points) of the address {}?
Note: Shares remaining: {}, please make sure the end sum amounts to 100% (i.e. 10% -> 1000 BPS)",
                    address, shares_remaining
                ))
                .validate_with(bps_validator)
                .interact()
                .unwrap()
                .parse::<u64>()
                .unwrap();

            if share <= shares_remaining {
                shares_remaining -= share;
                break share;
            } else {
                println!("The amount {} provided surpasses the amount remaining of {}.", share, shares_remaining);
            }
        };

        beneficiaries.push(address.clone());
        shares.insert(Share::new(Address::new(&address)?, share));
    }

    Ok(shares)
}

/// Collects the share of each address in royalty distribution in basis points.
///
/// # Arguments
/// * `addresses` - A slice of `Address` representing the addresses to collect shares for.
///
/// # Returns
/// * `Vec<u64>` - A vector of shares in basis points for each address.
pub fn royalty_shares(addresses: &[Address]) -> Vec<u64> {
    let theme = get_dialoguer_theme();

    let shares = addresses.iter().map(|address| {
        Input::with_theme(&theme)
            .with_prompt(
                format!("What is the percentage share (in basis points) of the address {}? (i.e. 10% -> 1000 BPS)", address)
            )
            .validate_with(number_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            // TODO: Not use unwrap here
            .unwrap()
        })
        .collect::<Vec<u64>>();

    shares
}
