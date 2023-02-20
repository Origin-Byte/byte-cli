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
    #[error("This error should not occur and likely results from a bug")]
    UnreachableError,
    #[error("{0}")]
    UploadError(String),
}

pub fn contextualize(msg: String) -> GutenError {
    print!("{}", msg);
    GutenError::UnreachableError
}
