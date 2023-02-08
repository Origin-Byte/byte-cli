use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Royalties {
    pub policy: Option<RoyaltyPolicy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum RoyaltyPolicy {
    Proportional { shares: Vec<Share>, bps: u64 },
    Constant { shares: Vec<Share>, fee: u64 },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub address: String,
    pub share: u64,
}

impl Default for Royalties {
    fn default() -> Self {
        Royalties { policy: None }
    }
}

impl Royalties {
    pub fn has_royalties(&self) -> bool {
        self.policy.is_some()
    }

    pub fn write(&self) -> String {
        let policy = self.policy.expect("No royalty policy setup found");

        let (royalty_shares, royalty_strategy) = match policy {
            RoyaltyPolicy::Proportional { shares, bps } => (
                &shares,
                format!(
                    "royalty::add_proportional_royalty(&mut royalty, {});",
                    bps
                ),
            ),
            RoyaltyPolicy::Constant { shares, fee } => (
                &shares,
                format!(
                    "royalty::add_constant_royalty(&mut royalty, {});",
                    fee
                ),
            ),
        };

        let code = if royalty_shares.len() == 1 {
            format!(
                "let royalty = royalty::from_address({address}, ctx);\n",
                address = royalty_shares[0].address,
            )
        } else {
            let vecmap = String::from("let royalty = vec_map::empty();\n");

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
                .collect::<()>();

            vecmap.push_str("\n");

            vecmap
        };

        let add_domain = "royalty::add_royalty_domain(delegated_witness, &mut collection, royalty);\n";

        code.push_str(royalty_strategy.as_str());
        code.push_str(add_domain);

        code
    }
}
