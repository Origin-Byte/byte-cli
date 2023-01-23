pub mod mint;

use std::fmt::Display;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustSdkError {
    #[error("The tag provided is not supported")]
    SomeError,
}

impl RustSdkError {
    pub fn error(msg: impl Display) -> RustSdkError {
        println!("[MissingField] {}", msg);

        RustSdkError::SomeError
    }
}
