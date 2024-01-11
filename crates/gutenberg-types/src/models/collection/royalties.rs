use super::Address;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Enum representing different royalty policies.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum RoyaltyPolicy {
    /// Proportional royalty policy, represented by a set of shares and a base percentage.
    #[serde(rename_all = "camelCase")]
    Proportional {
        /// A set of shares defining the distribution of royalties.
        #[serde(default)]
        shares: BTreeSet<Share>,
        /// The base percentage for collection royalties, expressed in basis points.
        collection_royalty_bps: u64,
    },
}

impl RoyaltyPolicy {
    /// Constructs a new `RoyaltyPolicy` with specified shares and collection royalty basis points.
    ///
    /// # Arguments
    /// * `shares` - A set of `Share` defining the distribution of royalties.
    /// * `collection_royalty_bps` - The base percentage for collection royalties in basis points.
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

/// Struct representing a share in royalty distribution.
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
    /// The address associated with the share.
    pub address: Address,
    /// The share amount, expressed in basis points.
    pub share_bps: u64,
}

impl Share {
    /// Constructs a new `Share`.
    ///
    /// # Arguments
    /// * `address` - The address associated with the share.
    /// * `share` - The share amount in basis points.
    pub fn new(address: Address, share: u64) -> Share {
        Share {
            address,
            share_bps: share,
        }
    }
}

impl RoyaltyPolicy {
    /// Adds beneficiaries to the royalty policy.
    ///
    /// # Arguments
    /// * `beneficiaries` - A mutable set of `Share` to be added to the royalty policy.
    pub fn add_beneficiaries(&mut self, beneficiaries: &mut BTreeSet<Share>) {
        match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps: _,
            } => shares.append(beneficiaries),
        };
    }

    /// Adds beneficiaries to the royalty policy from vectors of addresses and shares.
    ///
    /// # Arguments
    /// * `beneficiaries_vec` - A slice of `Address` representing beneficiaries.
    /// * `shares_vec` - A slice of `u64` representing the shares for each beneficiary.
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
