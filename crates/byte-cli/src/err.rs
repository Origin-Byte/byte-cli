use aws_sdk_s3::error::PutObjectError;
use hex::FromHexError;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeYaml(#[from] serde_yaml::Error),
    #[error(transparent)]
    Hex(#[from] FromHexError),
    #[error(transparent)]
    VarError(#[from] std::env::VarError),
    #[error(transparent)]
    AwsWriteError(#[from] aws_sdk_s3::types::SdkError<PutObjectError>),
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error("There was an error with the path")]
    PathError,
    #[error("The collection configuration file seems to be missing")]
    MissingConfig,
    #[error("Address length must be 20 bytes")]
    InvalidAddressLength,
    #[error("The market type provided is invalid")]
    InvalidMarket,
}

pub fn invalid_path(path: &Path) -> CliError {
    print!("Invalid path {path:?}");
    CliError::PathError
}

pub fn no_path() -> CliError {
    print!("The path provided does not exist");
    CliError::PathError
}
