use super::Address;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

// TODO: Doesn't need to be enum anymore
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum RoyaltyPolicy {
    #[serde(rename_all = "camelCase")]
    Proportional {
        #[serde(default)]
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
}
