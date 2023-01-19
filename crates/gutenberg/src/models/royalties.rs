use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Royalties {
    pub shares: Vec<Share>,
    pub proportional: Option<u64>,
    pub constant: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Share {
    pub address: String,
    pub share: u64,
}

impl Royalties {
    pub fn has_royalties(&self) -> bool {
        self.proportional.is_some() | self.constant.is_some()
    }

    pub fn write(&self) -> String {
        let shares = &self.shares;

        let mut policy = format!("let royalty = ");
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

        policy
    }
}
