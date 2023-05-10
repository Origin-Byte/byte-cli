use std::collections::BTreeSet;

use anyhow::Result;

use super::{address_validator, bps_validator, number_validator, FromPrompt};
use crate::{
    consts::{ROYALTY_OPTIONS, ROYALTY_OPTIONS_, TX_SENDER_ADDRESS},
    prelude::get_dialoguer_theme,
};

use dialoguer::{Confirm, Input, Select};
use gutenberg::{
    models::settings::royalties::{RoyaltyPolicy, Share},
    Schema,
};

impl FromPrompt for RoyaltyPolicy {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let creators = &schema.collection.creators;
        let policy = get_policy_type()?;

        match policy {
            Some(mut policy) => {
                if are_royalty_owners_creators() {
                    let shares = royalty_shares(creators);
                    policy.add_beneficiary_vecs(creators, &shares);
                } else {
                    let mut beneficiaries = get_beneficiaries();
                    policy.add_beneficiaries(&mut beneficiaries);
                }
                Ok(Some(policy))
            }
            None => Ok(None),
        }
    }
}

pub fn get_policy_type() -> Result<Option<RoyaltyPolicy>, anyhow::Error> {
    let theme = get_dialoguer_theme();

    let policy_index = Select::with_theme(&theme)
        .with_prompt("Which royalty policies do you want on your collection? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
        .items(&ROYALTY_OPTIONS)
        .interact()
        .unwrap();

    match ROYALTY_OPTIONS_[policy_index] {
        "Proportional" => {
            // TODO: Check that the basis points do not surpass 100%
            let bps = Input::with_theme(&theme)
                .with_prompt(
                    "What is the proportional royalty fee in basis points?",
                )
                .validate_with(bps_validator)
                .interact()?
                .parse::<u64>()?;

            Ok(Some(RoyaltyPolicy::Proportional {
                shares: BTreeSet::new(),
                collection_royalty_bps: bps,
            }))
        }
        "Constant" => {
            let fee = Input::with_theme(&theme)
                .with_prompt("What is the constant royalty fee in MIST?")
                .validate_with(number_validator)
                .interact()?
                .parse::<u64>()?;

            Ok(Some(RoyaltyPolicy::Constant {
                shares: BTreeSet::new(),
                fee,
            }))
        }
        "None" => Ok(None),
        _ => unreachable!(),
    }
}

pub fn are_royalty_owners_creators() -> bool {
    let theme = get_dialoguer_theme();

    Confirm::with_theme(&theme)
        .with_prompt("Are the royalty beneficiary addresses the same as the creator addresses?")
        .interact()
        .unwrap()
}

pub fn get_beneficiaries() -> BTreeSet<Share> {
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

        // TODO: Check that sum of shares matches 100%
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
        shares.insert(Share::new(address, share));
    }

    shares
}

pub fn royalty_shares(addresses: &Vec<String>) -> Vec<u64> {
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
