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
        match self.policy {
            Some(policy) => match policy {
                RoyaltyPolicy::Proportional { shares, bps } => {
                    let shares = &policy.shares;
                    todo!()
                }
                RoyaltyPolicy::Constant { shares, fee } => {
                    todo!()
                }
            },
            None => {
                todo!()
            }
        }

        let mut policy = "let royalty = ".to_string();
        match shares.len() {
            0 => {
                policy.push_str("royalty::new_empty(ctx);\n");
            }
            1 => {
                let address = &shares[0].address;
                policy.push_str(&format!(
                    "royalty::from_address(@{address}, ctx);\n"
                ));
            }
            _ => panic!("Arbitrary royalty share allocations not supported"),
        };

        if let Some(proportional) = self.proportional {
            policy.push_str(&format!("        royalty::add_proportional_royalty(&mut royalty, {proportional});\n"));
        }

        if let Some(constant) = self.constant {
            policy.push_str(&format!("        royalty::add_constant_royalty(&mut royalty, {constant});\n"));
        }

        policy.push_str("        royalty::add_royalty_domain(&mut collection, &mut mint_cap, royalty);\n");

        policy
    }
}
