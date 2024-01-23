use std::fmt;

use serde::{
    de::{self, Deserialize, Visitor},
    Serialize,
};

/// `MintCap` struct represents the minting capacity of an NFT.
/// It can be either limited to a specific supply or unlimited.
#[derive(Debug, Clone)]
pub struct MintCap {
    /// Optional field representing the supply cap for minting.
    /// If `None`, it indicates an unlimited minting capacity.
    pub supply: Option<u64>,
}

impl MintCap {
    /// Creates a `MintCap` with a specified supply limit.
    ///
    /// # Arguments
    /// * `supply` - A u64 value indicating the maximum number of tokens that can be minted.
    ///
    /// # Returns
    /// A `MintCap` instance with a defined supply limit.
    pub fn limited(supply: u64) -> Self {
        Self {
            supply: Some(supply),
        }
    }

    /// Creates a `MintCap` without a supply limit, indicating unlimited minting capacity.
    ///
    /// # Returns
    /// A `MintCap` instance with no supply limit.
    pub fn unlimited() -> Self {
        Self { supply: None }
    }
}

/// Implementation of `Deserialize` trait for `MintCap` to facilitate deserialization.
impl<'de> Deserialize<'de> for MintCap {
    /// Deserializes a `MintCap` from a deserializer.
    ///
    /// # Arguments
    /// * `deserializer` - The deserializer to use.
    ///
    /// # Returns
    /// * `Result<Self, D::Error>` - Result object containing `MintCap` if successful or an error.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct MintCapVisitor {}

        impl<'de> Visitor<'de> for MintCapVisitor {
            type Value = MintCap;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "u64 integer or \"unlimited\"")
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match s {
                    "unlimited" => Ok(MintCap::unlimited()),
                    _ => Err(E::invalid_value(
                        de::Unexpected::Str(s),
                        &"u64 integer or \"unlimited\"",
                    )),
                }
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MintCap::limited(v))
            }

            // TODO: Not sure if this is something that we should assume for
            // the user as its quite consequential
            //
            // fn visit_none<E>(self) -> Result<Self::Value, E>
            // where
            //     E: de::Error,
            // {
            //     println!("Expected u64 integer or \"unlimited\" for `nft.mintCap`, assuming \"unlimited\"");
            //     Ok(MintCap::unlimited())
            // }
        }

        deserializer.deserialize_any(MintCapVisitor {})
    }
}

/// Implementation of `Serialize` trait for `MintCap` to facilitate serialization.
impl Serialize for MintCap {
    /// Serializes the `MintCap` instance to a serializer.
    ///
    /// # Arguments
    /// * `serializer` - The serializer to use.
    ///
    /// # Returns
    /// * `Result<S::Ok, S::Error>` - Result of the serialization process.
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.supply {
            Some(supply) => serializer.serialize_u64(supply),
            None => serializer.serialize_str("unlimited"),
        }
    }
}
