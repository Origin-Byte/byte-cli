use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::err::{self, GutenError};

pub mod collection;
pub mod launchpad;
pub mod nft;

// TODO: Custom deserialize that validates address
#[derive(
    Debug,
    Serialize,
    Deserialize,
    Default,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    Clone,
)]
pub struct Address(String);

impl Address {
    pub fn new(address: String) -> Result<Self, GutenError> {
        Ok(Address(Address::validate_address(address)?))
    }

    fn validate_address(input: String) -> Result<String, GutenError> {
        let hexa_str = input.strip_prefix("0x").unwrap_or(&input);
        let hexa = hex::decode(hexa_str).map_err(|err| {
            err::invalid_address(
                hexa_str.to_string(),
                format!(
                    "Failed with the following error: {}
    Failed to decode hexadecimal string `{}`",
                    err, hexa_str,
                ),
            )
        })?;

        if hexa.len() != 32 {
            Err(err::invalid_address(
                hexa_str.to_string(),
                format!(
                    "Invalid Hexadecimal number. Expected 32 digits, got {}",
                    hexa.len(),
                ),
            ))
        } else {
            Ok(hex::encode(hexa))
        }
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}
