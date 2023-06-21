use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct MintEffects {
    pub minted_nfts: Vec<String>,
    pub error_logs: Vec<MintError>,
}

#[derive(Deserialize, Serialize)]
pub struct Minted(pub Vec<u32>);

impl Minted {
    pub fn new() -> Self {
        Minted(vec![])
    }
}

impl MintEffects {
    pub fn new() -> Self {
        Self {
            minted_nfts: vec![],
            error_logs: vec![],
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct MintError {
    from_nft: u32,
    to_nft: u32,
    error: String,
}

impl MintError {
    pub fn new(from_nft: u32, to_nft: u32, error: String) -> Self {
        Self {
            from_nft,
            to_nft,
            error,
        }
    }
}
