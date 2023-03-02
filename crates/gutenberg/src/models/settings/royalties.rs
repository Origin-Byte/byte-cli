use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, str::FromStr};

use crate::models::Address;

#[derive(Debug, Serialize, Deserialize)]
pub enum RoyaltyPolicy {
    Proportional { shares: BTreeSet<Share>, bps: u64 },
    Constant { shares: BTreeSet<Share>, fee: u64 },
}

#[derive(
    Debug, Default, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord,
)]
pub struct Share {
    pub address: Address,
    pub share: u64,
}

impl Share {
    pub fn new(address: Address, share: u64) -> Share {
        Share { address, share }
    }
}

impl FromStr for RoyaltyPolicy {
    type Err = ();

    fn from_str(input: &str) -> Result<RoyaltyPolicy, Self::Err> {
        match input {
            "Proportional" => Ok(RoyaltyPolicy::Proportional {
                shares: BTreeSet::default(),
                bps: u64::default(),
            }),
            "Constant" => Ok(RoyaltyPolicy::Constant {
                shares: BTreeSet::default(),
                fee: u64::default(),
            }),
            _ => Err(()),
        }
    }
}

impl RoyaltyPolicy {
    pub fn add_beneficiaries(&mut self, beneficiaries: &mut BTreeSet<Share>) {
        match self {
            RoyaltyPolicy::Proportional { shares, bps: _ } => {
                shares.append(beneficiaries)
            }
            RoyaltyPolicy::Constant { shares, fee: _ } => {
                shares.append(beneficiaries)
            }
        };
    }

    pub fn add_beneficiary_vecs(
        &mut self,
        creators_vec: &Vec<Address>,
        shares_vec: &Vec<u64>,
    ) {
        let push_creator =
            |creators_vec: &Vec<Address>, shares: &mut BTreeSet<Share>| {
                creators_vec
                    .iter()
                    .zip(shares_vec.iter())
                    .map(|(address, share)| {
                        shares.insert(Share::new(address.clone(), *share))
                    })
                    .for_each(drop);
            };

        let shares = match self {
            RoyaltyPolicy::Proportional { shares, bps: _ } => shares,
            RoyaltyPolicy::Constant { shares, fee: _ } => shares,
        };

        push_creator(creators_vec, shares);
    }

    pub fn write_domain(&self) -> String {
        let (royalty_shares, royalty_strategy) = match self {
            RoyaltyPolicy::Proportional { shares, bps } => (
                shares.clone(),
                format!(
                    "        nft_protocol::royalty::add_proportional_royalty(&mut royalty, {});\n",
                    bps
                ),
            ),
            RoyaltyPolicy::Constant { shares, fee } => (
                shares.clone(),
                format!(
                    "        nft_protocol::royalty::add_constant_royalty(&mut royalty, {});\n",
                    fee
                ),
            ),
        };

        let mut code = if royalty_shares.len() == 1 {
            format!(
                "let royalty = nft_protocol::royalty::from_address({address}, ctx);\n",
                address = royalty_shares.first().unwrap().address,
            )
        } else {
            let mut vecmap = String::from(
                "\n        let royalty_map = sui::vec_map::empty();\n",
            );

            royalty_shares
                .iter()
                .map(|share| {
                    vecmap.push_str(
                        format!(
                        "        sui::vec_map::insert(&mut royalty_map, {address}, {share});\n",
                        address = share.address,
                        share = share.share
                    )
                        .as_str(),
                    );
                })
                .for_each(drop);

            vecmap.push_str("\n");
            vecmap.push_str(
                "        let royalty = nft_protocol::royalty::from_shares(royalty_map, ctx);\n",
            );

            vecmap
        };

        let add_domain = "        nft_protocol::royalty::add_royalty_domain(delegated_witness, &mut collection, royalty);\n";

        code.push_str(royalty_strategy.as_str());
        code.push_str(add_domain);

        code
    }

    pub fn write_entry_fn(&self, witness: &String) -> String {
        let domain = match self {
            RoyaltyPolicy::Proportional { shares: _, bps: _ } => {
                "calculate_proportional_royalty(domain, sui::balance::value(b))"
            }
            RoyaltyPolicy::Constant { shares: _, fee: _ } => {
                "calculate_constant_royalty(domain)"
            }
        };

        format!(
            "\n
    /// Calculates and transfers royalties to the `RoyaltyDomain`
    public entry fun collect_royalty<FT>(
        payment: &mut nft_protocol::royalties::TradePayment<{witness}, FT>,
        collection: &mut nft_protocol::collection::Collection<{witness}>,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let b = nft_protocol::royalties::balance_mut(Witness {{}}, payment);

        let domain = nft_protocol::royalty::royalty_domain(collection);
        let royalty_owed =
            nft_protocol::royalty::{domain};

        nft_protocol::royalty::collect_royalty(collection, b, royalty_owed);
        nft_protocol::royalties::transfer_remaining_to_beneficiary(Witness {{}}, payment, ctx);
    }}"
        )
    }
}
