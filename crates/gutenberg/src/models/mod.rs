use std::{
    fmt::{self, Display},
    marker::PhantomData,
};

use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::err::{self, GutenError};

pub mod collection;
pub mod launchpad;
pub mod nft;
pub mod settings;

// TODO: Custom deserialize that validates address
#[derive(
    Debug, Serialize, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Hash,
)]
pub struct Address(String);

impl Address {
    pub fn new(address: String) -> Result<Self, GutenError> {
        Ok(Address(Address::validate_address(address)?))
    }

    pub fn as_string(&self) -> &String {
        &self.0
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

struct AddressVisitor {
    marker: PhantomData<fn() -> Address>,
}

impl AddressVisitor {
    fn new() -> Self {
        AddressVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for AddressVisitor {
    type Value = Address;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "The address provided is not valid.")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let sanitized = Address::validate_address(s.to_string())
            .expect(format!("Failed to parse address {}", s,).as_str());

        Ok(Address(sanitized))
    }
}

// This is the trait that informs Serde how to deserialize Version.
impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate VersionVisitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of Version.
        deserializer.deserialize_str(AddressVisitor::new())
    }
}
