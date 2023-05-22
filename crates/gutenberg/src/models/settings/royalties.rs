use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, str::FromStr};

use crate::models::Address;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoyaltyPolicy {
    #[serde(rename_all = "camelCase")]
    Proportional {
        shares: BTreeSet<Share>,
        collection_royalty_bps: u64,
    },
}

#[derive(
    Debug, Default, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord,
)]
#[serde(rename_all = "camelCase")]
pub struct Share {
    pub address: Address,
    pub share_bps: u64,
}

impl Share {
    pub fn new(address: Address, share: u64) -> Share {
        Share {
            address,
            share_bps: share,
        }
    }
}

impl FromStr for RoyaltyPolicy {
    type Err = ();

    fn from_str(input: &str) -> Result<RoyaltyPolicy, Self::Err> {
        match input {
            "proportional" => Ok(RoyaltyPolicy::Proportional {
                shares: BTreeSet::default(),
                collection_royalty_bps: u64::default(),
            }),
            _ => Err(()),
        }
    }
}

impl RoyaltyPolicy {
    pub fn add_beneficiaries(&mut self, beneficiaries: &mut BTreeSet<Share>) {
        match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps: _,
            } => shares.append(beneficiaries),
        };
    }

    pub fn add_beneficiary_vecs(
        &mut self,
        beneficiaries_vec: &Vec<Address>,
        shares_vec: &Vec<u64>,
    ) {
        let push_beneficiary =
            |beneficiaries_vec: &Vec<Address>, shares: &mut BTreeSet<Share>| {
                beneficiaries_vec
                    .iter()
                    .zip(shares_vec.iter())
                    .map(|(address, share)| {
                        shares.insert(Share::new(address.clone(), *share))
                    })
                    .for_each(drop);
            };

        let shares = match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps: _,
            } => shares,
        };

        push_beneficiary(beneficiaries_vec, shares);
    }

    pub fn write_strategy(&self) -> String {
        let (royalty_shares, royalty_strategy) = match self {
            RoyaltyPolicy::Proportional { shares, collection_royalty_bps: bps } => (
                shares.clone(),
                format!(
                    "        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            {},
            ctx,
        );",
                    bps
                ),
            ),
        };

        let mut code = {
            let mut vecmap = String::from(
                "\n        let royalty_map = sui::vec_map::empty();\n",
            );

            royalty_shares
                .iter()
                .map(|share| {
                    vecmap.push_str(
                        format!(
                        "        sui::vec_map::insert(&mut royalty_map, @{address}, {share});\n",
                        share = share.share_bps,
                        address = share.address
                    )
                        .as_str(),
                    );
                })
                .for_each(drop);

            vecmap.push_str("\n");

            vecmap
        };

        code.push_str(royalty_strategy.as_str());

        code
    }
}
