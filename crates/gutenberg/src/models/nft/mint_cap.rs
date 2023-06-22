use std::fmt;

use serde::{
    de::{self, Deserialize, Visitor},
    Serialize,
};

#[derive(Debug, Clone)]
pub struct MintCap {
    supply: Option<u64>,
}

impl MintCap {
    pub fn limited(supply: u64) -> Self {
        Self {
            supply: Some(supply),
        }
    }

    pub fn unlimited() -> Self {
        Self { supply: None }
    }

    /// Write MintCap instantiation
    pub fn write_move_init(&self, witness: &str, type_name: &str) -> String {
        let mint_cap_str = match self.supply {
            Some(supply) => format!("

        let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
            &witness, collection_id, {supply}, ctx
        );"),
            None =>
            format!("

        let mint_cap = nft_protocol::mint_cap::new_unlimited<{witness}, {type_name}>(
            &witness, collection_id, ctx
        );")
        };

        format!("{mint_cap_str}
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
    }
}

impl<'de> Deserialize<'de> for MintCap {
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

impl Serialize for MintCap {
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
