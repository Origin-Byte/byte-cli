use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, str::FromStr};

use crate::models::Address;

// TODO: Doesn't need to be enum anymore
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum RoyaltyPolicy {
    #[serde(rename_all = "camelCase")]
    Proportional {
        shares: BTreeSet<Share>,
        collection_royalty_bps: u64,
    },
}

impl RoyaltyPolicy {
    pub fn new(
        shares: BTreeSet<Share>,
        collection_royalty_bps: u64,
    ) -> RoyaltyPolicy {
        RoyaltyPolicy::Proportional {
            shares,
            collection_royalty_bps,
        }
    }
}

#[derive(
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
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
        beneficiaries_vec: &[Address],
        shares_vec: &[u64],
    ) {
        let shares = match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps: _,
            } => shares,
        };

        beneficiaries_vec.iter().zip(shares_vec.iter()).for_each(
            |(address, share)| {
                shares.insert(Share::new(address.clone(), *share));
            },
        );
    }

    pub fn write_move_init(&self) -> String {
        match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps,
            } => {
                let mut creators_str = "

        let royalty_map = sui::vec_map::empty();"
                    .to_string();

                for share in shares {
                    creators_str.push_str(&format!(
                        "
        sui::vec_map::insert(&mut royalty_map, @{address}, {share});",
                        share = share.share_bps,
                        address = share.address
                    ));
                }

                let domain = format!(
                    "

        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            {collection_royalty_bps},
            ctx,
        );"
                );
                creators_str.push_str(&domain);
                creators_str
            }
        }
    }
}
