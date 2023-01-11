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
    /// The royalty fees creators accumulate on the sale of NFTs
    pub royalty_fee_bps: String,
    /// Field for extra data
    pub url: String,
}

impl Collection {
    pub fn add_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn add_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn add_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }

    pub fn add_tag(&mut self, tag_string: String) -> Result<(), GutenError> {
        let tag = Tag::from_str(tag_string.as_str())
            .map_err(|_| GutenError::UnsupportedTag)?;

        self.tags.push(tag);

        Ok(())
    }

    pub fn add_royalty_fee_bps(&mut self, royalty_bps: String) {
        self.royalty_fee_bps = royalty_bps;
    }

    pub fn add_url(&mut self, royalty_bps: String) {
        self.royalty_fee_bps = royalty_bps;
    }
}
