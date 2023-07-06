//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.

mod royalties;
mod supply;
mod tags;

pub use super::address::Address;
use crate::deunicode;
pub use royalties::{RoyaltyPolicy, Share};
use serde::{Deserialize, Serialize};
pub use supply::Supply;
pub use tags::{Tag, Tags};

/// Contains the metadata fields of the collection
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    /// The name of the collection
    pub name: Option<String>,
    /// The description of the collection
    pub description: Option<String>,
    /// The symbol/ticker of the collection
    pub symbol: Option<String>,
    /// The URL of the collection website
    pub url: Option<String>,
    #[serde(default)]
    /// The addresses of creators
    pub creators: Vec<Address>,
    /// Collection tags
    pub tags: Option<Tags>,
    /// Collection-level supply
    #[serde(default)]
    pub supply: Supply,
    /// Collection royalties
    pub royalties: Option<RoyaltyPolicy>,
}

impl CollectionData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: Option<String>,
        description: Option<String>,
        symbol: Option<String>,
        url: Option<String>,
        creators: Vec<Address>,
        supply: Supply,
        royalties: Option<RoyaltyPolicy>,
        tags: Option<Tags>,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
            supply,
            royalties,
            tags,
        }
    }

    pub fn name(&self) -> Option<String> {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.name.as_ref().map(|name| deunicode(name))
    }

    pub fn description(&self) -> Option<String> {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.description
            .as_ref()
            .map(|description| deunicode(description))
    }

    pub fn url(&self) -> Option<String> {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.url.clone()
    }

    pub fn symbol(&self) -> Option<String> {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.symbol.clone()
    }

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn royalties(&self) -> &Option<RoyaltyPolicy> {
        &self.royalties
    }

    pub fn creators(&self) -> &Vec<Address> {
        &self.creators
    }

    pub fn tags(&self) -> &Option<Tags> {
        &self.tags
    }

    /// Whether collection has royalty policy defined
    pub fn has_royalties(&self) -> bool {
        self.royalties().is_some()
    }

    /// Whether `&mut Collection` needs to be passed into mint methods
    pub fn requires_collection(&self) -> bool {
        self.supply().requires_collection()
    }

    /// Disables features that should not be enabled in demo mode
    pub fn enforce_demo(&mut self) {
        self.supply = Supply::Untracked;
        self.royalties = None;
    }
}
