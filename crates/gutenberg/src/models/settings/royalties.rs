use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, str::FromStr};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RoyaltyPolicy {
    Proportional { shares: BTreeSet<Share>, bps: u64 },
}

#[derive(
    Debug, Default, Serialize, Deserialize, PartialEq, PartialOrd, Eq, Ord,
)]
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
            "proportional" => Ok(RoyaltyPolicy::Proportional {
                shares: BTreeSet::default(),
                bps: u64::default(),
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
        };
    }

    pub fn add_beneficiary_vecs(
        &mut self,
        beneficiaries_vec: &Vec<String>,
        shares_vec: &Vec<u64>,
    ) {
        let push_beneficiary =
            |beneficiaries_vec: &Vec<String>, shares: &mut BTreeSet<Share>| {
                beneficiaries_vec
                    .iter()
                    .zip(shares_vec.iter())
                    .map(|(address, share)| {
                        shares.insert(Share::new(address.clone(), *share))
                    })
                    .for_each(drop);
            };

        let shares = match self {
            RoyaltyPolicy::Proportional { shares, bps: _ } => shares,
        };

        push_beneficiary(beneficiaries_vec, shares);
    }

    pub fn write_strategy(&self) -> String {
        let (royalty_shares, royalty_strategy) = match self {
            RoyaltyPolicy::Proportional { shares, bps } => (
                shares.clone(),
                format!(
                    "        royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            {},
            ctx,
        );\n",
                    bps
                ),
            ),
        };

        let (mut code, kiosk_init) = {
            let mut vecmap = String::from(
                "\n        let royalty_map = sui::vec_map::empty();\n",
            );

            let mut kiosk_init = "".to_string();

            royalty_shares
                .iter()
                .map(|share| {
                    // TODO: Check if valid address instead
                    let address = if share.address == "sui::tx_context::sender(ctx)" {
                        share.address.clone()
                    } else {
                        format!("@{address}", address = share.address)
                    };

                    vecmap.push_str(
                        format!(
                        "        sui::vec_map::insert(&mut royalty_map, {address}, {share});\n",
                        share = share.share
                    )
                        .as_str(),
                    );

                    kiosk_init.push_str(
                        format!(
                        "        ob_kiosk::ob_kiosk::init_for_address({address}, ctx);\n"
                    )
                        .as_str(),
                    );
                })
                .for_each(drop);

            vecmap.push_str("\n");

            (vecmap, kiosk_init)
        };

        code.push_str(royalty_strategy.as_str());
        code.push_str(
            "\n       
        // Setup Kiosks for royalty address(es)\n",
        );

        code.push_str(kiosk_init.as_str());

        code
    }
}
