//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::models::tags::Tags;

use serde::{Deserialize, Serialize};

/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Collection {
    /// The name of the collection
    pub name: String,
    /// The description of the collection
    pub description: String,
    /// The symbol/ticker of the collection
    pub symbol: String,
    /// A set of strings that categorize the domain in which the NFT operates
    pub tags: Tags,
    /// Field for extra data
    pub url: Option<String>,
}

impl Collection {
    pub fn new(
        name: String,
        description: String,
        symbol: String,
        tags: Tags,
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

    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = tags;
    }
}
