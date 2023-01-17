use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("An IO error has occured")]
    IoError(#[from] std::io::Error),
    // #[error("The tag provided is not supported")]
    // UnsupportedTag,
    #[error("The address provided is invalid")]
    InvalidAddress,
    #[error("The market type provided is invalid")]
    InvalidMarket,
}
