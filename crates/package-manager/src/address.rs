use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt::{self, Display};

#[derive(thiserror::Error, PartialEq, Debug)]
pub enum AddressError {
    #[error("Invalid address encoding, expected hexadecimal, got {0}")]
    InvalidEncoding(String),
    #[error("Invalid address length, expected 32 bytes, got {0}")]
    InvalidLength(usize),
}

#[derive(Debug, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Hash)]
pub struct Address(String);

impl Address {
    pub fn new(address: &str) -> Result<Self, AddressError> {
        // Extract byte string and pad to 64 bytes
        let hexa_str = address.strip_prefix("0x").unwrap_or(&address);

        // Pad address with zeroes if less than 64 characters
        let hexa_str = format!("{:0>64}", hexa_str);
        let address = hexa_str
            .chars()
            .all(|char| char.is_ascii_hexdigit())
            .then_some(hexa_str)
            .ok_or_else(|| {
                AddressError::InvalidEncoding(address.to_string())
            })?;

        let length = address.len();
        (length <= 64)
            .then_some(Address(address))
            .ok_or(AddressError::InvalidLength(length))
    }

    pub fn zero() -> Self {
        Address("0".to_string())
    }

    pub fn as_string(&self) -> &String {
        &self.0
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x{}", &self.0)
    }
}

// This is the trait that informs Serde how to deserialize Version.
impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AddressVisitor;

        impl<'de> Visitor<'de> for AddressVisitor {
            type Value = Address;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> fmt::Result {
                write!(formatter, "32 byte hex-encoded string")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Address::new(s).map_err(|err| match err {
                    AddressError::InvalidEncoding(s) => {
                        de::Error::invalid_value(
                            Unexpected::Str(&s),
                            &"hex-encoded string",
                        )
                    }
                    AddressError::InvalidLength(len) => {
                        de::Error::invalid_length(
                            len,
                            &"32-byte hex-encoded string",
                        )
                    }
                })
            }
        }

        // Instantiate VersionVisitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of Version.
        deserializer.deserialize_str(AddressVisitor {})
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("0x{}", self.0))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn input_addresses() {
        // In converting a string to an address the function should be
        // able to handle hexadecimals with or without 0x prefix.
        let address =
            "0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df";
        let address_2 =
            "1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df";

        assert_eq!(
            Address::new(address).unwrap(),
            Address::new(address_2).unwrap()
        );
    }

    #[test]
    fn adds_padding() {
        // 1.
        let actual =
            Address::new("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1432144152")
                .unwrap();

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "00000000000000000d8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1432144152",
        )
        .unwrap();

        assert_eq!(actual.0, expected.0);

        // 2.

        let actual = Address::new("0x0").unwrap();

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();

        assert_eq!(actual.0, expected.0);

        let actual = Address::new("0x").unwrap();
        assert_eq!(actual.0, expected.0);

        let actual = Address::new("").unwrap();
        assert_eq!(actual.0, expected.0);

        // 3.

        let actual = Address::new("0x2").unwrap();

        assert_eq!(actual.0.len(), 64);

        let expected = Address::new(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();

        assert_eq!(actual.0, expected.0);
    }

    #[test]
    fn err_hex_too_big() {
        assert_eq!(
            Address::new("0x30f1ee29f5d8763a75c042122eaa795fb60c25ca256c5ee469a57c33590c59d3c5ee469a57c335").unwrap_err(),
            AddressError::InvalidLength(78),
        );
    }

    #[test]
    fn err_not_an_hex() {
        assert_eq!(
            Address::new("0x30f1ee29f5d8763a75c042122eaa795fb60z25ca256c5ee469a57c33590c59d3c5ee469a57c335").unwrap_err(),
            AddressError::InvalidEncoding("0x30f1ee29f5d8763a75c042122eaa795fb60z25ca256c5ee469a57c33590c59d3c5ee469a57c335".to_string()),
        );
    }
}
