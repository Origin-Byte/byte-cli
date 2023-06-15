#[cfg(feature = "full")]
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct MintCap {
    supply: Option<u64>,
}

#[cfg(feature = "full")]
impl MintCap {
    pub fn new(supply: Option<u64>) -> Self {
        Self { supply }
    }

    /// Write MintCap instantiation
    pub fn write_move_init(&self, witness: &str, type_name: &str) -> String {
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

        format!("{mint_cap_str}
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
    }
}

#[cfg(not(feature = "full"))]
/// Write MintCap instantiation with supply always limited to 100
pub fn write_move_init(witness: &str, type_name: &str) -> String {
    format!("

        let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
            &witness, collection_id, 100, ctx
        );
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
}
