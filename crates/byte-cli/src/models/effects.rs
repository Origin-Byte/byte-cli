use serde::{Deserialize, Serialize};

/// `MintEffects` struct holds the results of a minting operation.
///
/// It contains a list of successfully minted NFTs and a collection of errors that occurred during the minting process.
#[derive(Deserialize, Serialize, Default)]
pub struct MintEffects {
    /// A vector of strings representing the identifiers of the successfully minted NFTs.
    pub minted_nfts: Vec<String>,
    /// A vector of `MintError` instances, each representing an error that occurred during the minting process.
    pub error_logs: Vec<MintError>,
}

/// `Minted` struct is a wrapper around a vector of unsigned 32-bit integers.
///
/// It is primarily used to represent a collection of minted items, each identified by a unique ID.
#[derive(Deserialize, Serialize, Default)]
pub struct Minted(pub Vec<u32>);

/// `MintError` struct represents an error that occurred during the minting process of NFTs.
///
/// It contains information about the source NFT, the target NFT, and a description of the error.
#[derive(Deserialize, Serialize)]
pub struct MintError {
    /// The identifier of the source NFT from which the error originated.
    from_nft: u32,
    /// The identifier of the target NFT that was involved in the error.
    to_nft: u32,
    /// A string describing the nature of the error.
    error: String,
}

impl MintError {
    /// Constructs a new `MintError`.
    ///
    /// # Arguments
    ///
    /// * `from_nft` - The identifier of the source NFT.
    /// * `to_nft` - The identifier of the target NFT.
    /// * `error` - A string describing the error.
    ///
    /// # Returns
    ///
    /// Returns a new instance of `MintError`.
    pub fn new(from_nft: u32, to_nft: u32, error: String) -> Self {
        Self {
            from_nft,
            to_nft,
            error,
        }
    }
}
