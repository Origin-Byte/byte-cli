use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum RoyaltyPolicy {
    Proportional { shares: Vec<Share>, bps: u64 },
    Constant { shares: Vec<Share>, fee: u64 },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Share {
    pub address: String,
    pub share: u64,
}

impl Share {
    pub fn new(address: String, share: u64) -> Share {
        Share { address, share }
    }
}

impl FromStr for RoyaltyPolicy {
    type Err = ();

    fn from_str(input: &str) -> Result<RoyaltyPolicy, Self::Err> {
        match input {
            "Proportional" => Ok(RoyaltyPolicy::Proportional {
                shares: Vec::default(),
                bps: u64::default(),
            }),
            "Constant" => Ok(RoyaltyPolicy::Constant {
                shares: Vec::default(),
                fee: u64::default(),
            }),
            _ => Err(()),
        }
    }
}

impl RoyaltyPolicy {
    pub fn add_beneficiaries(&mut self, beneficiaries: &mut Vec<Share>) {
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
        creators_vec: &Vec<String>,
        shares_vec: &Vec<u64>,
    ) {
        let push_creator =
            |creators_vec: &Vec<String>, shares: &mut Vec<Share>| {
                creators_vec
                    .iter()
                    .zip(shares_vec.iter())
                    .map(|(address, share)| {
                        shares.push(Share::new(address.clone(), share.clone()))
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
                    "royalty::add_proportional_royalty(&mut royalty, {});",
                    bps
                ),
            ),
            RoyaltyPolicy::Constant { shares, fee } => (
                shares.clone(),
                format!(
                    "royalty::add_constant_royalty(&mut royalty, {});",
                    fee
                ),
            ),
        };

        let mut code = if royalty_shares.len() == 1 {
            format!(
                "let royalty = royalty::from_address({address}, ctx);\n",
                address = royalty_shares[0].address,
            )
        } else {
            let mut vecmap = String::from("let royalty = vec_map::empty();\n");

            royalty_shares
                .iter()
                .map(|share| {
                    vecmap.push_str(
                        format!(
                        "vec_map::insert(&mut royalty, {address}, {share});\n",
                        address = share.address,
                        share = share.share
                    )
                        .as_str(),
                    );
                })
                .for_each(drop);

            vecmap.push_str("\n");

            vecmap
        };

        let add_domain = "royalty::add_royalty_domain(delegated_witness, &mut collection, royalty);\n";

        code.push_str(royalty_strategy.as_str());
        code.push_str(add_domain);

        code
    }

    pub fn write_entry_fn(&self, name: &String) -> String {
        let domain = match self {
            RoyaltyPolicy::Proportional { shares: _, bps: _ } => {
                "calculate_proportional_royalty(domain, balance::value(b))"
            }
            RoyaltyPolicy::Constant { shares: _, fee: _ } => {
                "calculate_constant_royalty(domain)"
            }
        };

        format!(
            "\n
            /// Calculates and transfers royalties to the `RoyaltyDomain`
            public entry fun collect_royalty<FT>(
                payment: &mut TradePayment<{name}, FT>,
                collection: &mut Collection<{name}>,
                ctx: &mut TxContext,
            ) {{
                let b = royalties::balance_mut(Witness {{}}, payment);

                let domain = royalty::royalty_domain(collection);
                let royalty_owed =
                    royalty::{domain};

                royalty::collect_royalty(collection, b, royalty_owed);
                royalties::transfer_remaining_to_beneficiary(Witness {{}}, payment, ctx);
            }}\n",
            name = name,
            domain = domain,
        )
    }
}
