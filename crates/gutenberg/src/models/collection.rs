//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::err::GutenError;
use crate::types::Tag;

use serde::Deserialize;
use std::str::FromStr;

/// Contains the metadata fields of the collection
#[derive(Debug, Deserialize)]
pub struct Collection {
    /// The name of the collection
    pub name: String,
    /// The description of the collection
    pub description: String,
    /// The symbol/ticker of the collection
    pub symbol: String,
    /// A set of strings that categorize the domain in which the NFT operates
    pub tags: Vec<Tag>,
    /// Field for extra data
    pub url: Option<String>,
}

impl Collection {
    pub fn new() -> Collection {
        Collection {
            name: String::new(),
            description: String::new(),
            symbol: String::new(),
            tags: Vec::new(),
            url: Option::None,
        }
    }

    pub fn new_from(
        name: String,
        description: String,
        symbol: String,
        tags: Vec<Tag>,
        url: String,
    ) -> Collection {
        Collection {
            name,
            description,
            symbol,
            tags,
            url: Option::Some(url),
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }

    pub fn set_url(&mut self, symbol: String) {
        self.symbol = symbol;
    }

    pub fn set_tags(&mut self, tags: &Vec<String>) -> Result<(), GutenError> {
        self.tags = tags
            .iter()
            .map(|string| {
                Tag::from_str(string).map_err(|_| GutenError::UnsupportedTag)
            })
            .collect::<Result<Vec<Tag>, GutenError>>()?;

        Ok(())
    }

    pub fn push_tag(&mut self, tag_string: String) -> Result<(), GutenError> {
        let tag = Tag::from_str(tag_string.as_str())
            .map_err(|_| GutenError::UnsupportedTag)?;

        self.tags.push(tag);

        Ok(())
    }

    // TODO
    pub fn pop_tag(&mut self, _tag_string: String) {}

    // pub fn set_royalty_fee_bps(&mut self, royalty_bps: String) {
    //     self.royalty_fee_bps = royalty_bps;
    // }

    // pub fn set_url(&mut self, royalty_bps: String) {
    //     self.royalty_fee_bps = royalty_bps;
    // }
}
