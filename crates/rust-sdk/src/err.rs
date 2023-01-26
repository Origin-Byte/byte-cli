use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustSdkError {
    #[error("The tag provided is not supported")]
    SomeError,
    #[error(transparent)]
    SuiSdkError(#[from] sui_sdk::error::Error),
    // Occurs in Sui
    #[error(transparent)]
    AnyhowError(#[from] anyhow::Error),
    #[error(transparent)]
    SuiTypesError(#[from] sui_types::error::SuiError),
    #[error("A signature error has occurred")]
    SignatureError(#[from] signature::Error),
    #[error("Unable to parse object ID")]
    ObjectIDParseError,
}

pub fn object_id(err: impl Display, objet_str: &str) -> RustSdkError {
    println!(r#"Could not get ObjectID from &str "{}": {err}"#, objet_str);

    RustSdkError::ObjectIDParseError
}
