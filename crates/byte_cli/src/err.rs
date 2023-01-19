use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    Hex(#[from] FromHexError),
    // #[error("The tag provided is not supported")]
    // UnsupportedTag,
    #[error("Address length must be 20 bytes")]
    InvalidAddressLength,
    #[error("The market type provided is invalid")]
    InvalidMarket,
}
