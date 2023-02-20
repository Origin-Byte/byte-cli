use thiserror::Error;

#[derive(Error, Debug)]
pub enum GutenError {
    #[error("An IO error has occured")]
    IoError(#[from] std::io::Error),
    #[error("{0}")]
    UnsupportedCollectionInput(String),
    #[error("{0}")]
    UnsupportedNftInput(String),
    #[error("{0}")]
    UnsupportedSettings(String),
    #[error("The address `{0}` provided is not valid.")]
    InvalidAddress(String),
    #[error("The tag provided is not supported")]
    UnsupportedTag,
    #[error("Unsupported Collection Symbol")]
    UnsupportedSymbol,
    #[error("This error should not occur and likely results from a bug")]
    UnreachableError,
    #[error("{0}")]
    UploadError(String),
}

pub fn contextualize(msg: String) -> GutenError {
    print!("{}", msg);
    GutenError::UnreachableError
}

pub fn invalid_address(address: String, msg: String) -> GutenError {
    print!("{}", msg);
    GutenError::UnsupportedSettings(address)
}
