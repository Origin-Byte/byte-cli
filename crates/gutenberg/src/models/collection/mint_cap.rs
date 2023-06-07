use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MintCap {
    supply: Option<u64>,
}

impl MintCap {
    pub fn new(supply: Option<u64>) -> Self {
        Self { supply }
    }

    pub fn write_move_init(&self, witness: &str, type_name: &str) -> String {
        let mut init_str = String::new();

        let mint_cap_str = match self.supply {
            Some(supply) => format!("

        let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
            &witness, collection_id, {supply}, ctx
        );"),
            None =>
            format!("

        let mint_cap = nft_protocol::mint_cap::new_unlimited<{witness}, {type_name}>(
            witness, collection_id, ctx
        );")
        };

        init_str.push_str(&mint_cap_str);
        init_str.push_str(
            "
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));",
        );

        init_str
    }
}
