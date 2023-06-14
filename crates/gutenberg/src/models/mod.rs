use std::{
    fmt::{self, Display},
    marker::PhantomData,
};

use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};

use crate::err::{self, GutenError};

pub mod collection;
pub mod launchpad;
pub mod nft;

// TODO: Custom deserialize that validates address
#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Hash)]
pub struct Address(String);

impl Address {
    pub fn new(address: String) -> Result<Self, GutenError> {
        Ok(Address(Address::validate_address(address)?))
    }

    pub fn as_string(&self) -> &String {
        &self.0
    }

    fn validate_address(input: String) -> Result<String, GutenError> {
        let mut hexa_str = input.strip_prefix("0x").unwrap_or(&input);
        let padded_input = Self::add_padding(hexa_str)?;

        if hexa_str.len() < 64 {
            // Adding padding
            hexa_str = padded_input.as_str();
        }

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

    fn add_padding(hex_string: &str) -> Result<String, GutenError> {
        if hex_string.len() > 64 {
            return Err(err::invalid_address(
                hex_string.to_string(),
                format!(
                    "Failed with the following error:
        Hexadecimal number too big. It cannot surpass 64 digits, instead if has`{}`",
                    hex_string,
                ),
            ));
        }

        let padding_length = 64 - hex_string.len();
        let padding = "0".repeat(padding_length);

        Ok(format!("{}{}", padding, hex_string))
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", &self.0)
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
        write!(formatter, "Not valid 32-byte hex-encoded string")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let sanitized = Address::validate_address(s.to_string())
            .unwrap_or_else(|_| panic!("Failed to parse address {}", s));

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

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut addr = String::from("0x");
        addr.push_str(self.0.as_str());

        serializer.serialize_str(addr.as_str())
    }
}

#[cfg(test)]
mod test {
    use super::Address;
    use crate::err::GutenError;

    #[test]
    fn input_addresses() -> Result<(), GutenError> {
        // In converting a string to an address the function should be
        // able to handle hexadecimals with or without 0x prefix.
        let address =
        "0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df"
            .to_string();
        let address_2 =
            "1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df"
                .to_string();

        assert_eq!(Address::new(address)?, Address::new(address_2)?);

        Ok(())
    }

    #[test]
    fn adds_padding() -> Result<(), GutenError> {
        // 1.
        let actual = Address::new(
            "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1432144152".to_string(),
        )?;

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "00000000000000000d8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1432144152"
                .to_string(),
        )?;

        assert_eq!(actual.0, expected.0);

        // 2.

        let actual = Address::new("0x0".to_string())?;

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        )?;

        assert_eq!(actual.0, expected.0);

        let actual = Address::new("0x".to_string())?;
        assert_eq!(actual.0, expected.0);

        let actual = Address::new("".to_string())?;
        assert_eq!(actual.0, expected.0);

        // 3.

        let actual = Address::new("0x2".to_string())?;

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "0000000000000000000000000000000000000000000000000000000000000002"
                .to_string(),
        )?;

        assert_eq!(actual.0, expected.0);

        Ok(())
    }

    #[test]
    fn err_hex_too_big() -> Result<(), GutenError> {
        assert!(Address::new(
            "0x30f1ee29f5d8763a75c042122eaa795fb60c25ca256c5ee469a57c33590c59d3c5ee469a57c335"
                .to_string()
        )
        .is_err());
        // assert!(Address::new("This is not an hexadecimal".to_string()).is_err());

        Ok(())
    }

    #[test]
    fn err_not_an_hex() -> Result<(), GutenError> {
        assert!(Address::new(
            "0x30f1ee29f5d8763a75c042122eaa795fb60c25ca256c5ee469a57c33590c59d3c5ee469a57c335"
                .to_string()
        )
        .is_err());
        // assert!(Address::new("This is not an hexadecimal".to_string()).is_err());

        Ok(())
    }
}
