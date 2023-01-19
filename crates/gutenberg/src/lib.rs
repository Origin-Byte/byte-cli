pub mod models;
pub mod package;
mod schema;

pub use schema::Schema;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GutenError {
    #[error("An IO error has occured")]
    IoError(#[from] std::io::Error),
    #[error("The tag provided is not supported")]
    UnsupportedTag,
    #[error("The NFT field provided is not a supported")]
    UnsupportedNftField,
    #[error("The NFT behaviour provided is not a supported")]
    UnsupportedNftBehaviour,
    #[error("The Supply Policy provided is not a supported")]
    UnsupportedSupply,
    #[error("The Royalty Policy provided is not a supported")]
    UnsupportedRoyalty,
}
