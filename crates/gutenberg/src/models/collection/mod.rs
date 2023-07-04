//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.

mod royalties;
mod supply;
mod tags;

use crate::deunicode;

use package_manager::Address;
pub use royalties::{RoyaltyPolicy, Share};
pub use supply::Supply;
pub use tags::{Tag, Tags};

use serde::{Deserialize, Serialize};

/// Contains the metadata fields of the collection
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    /// The name of the collection
    name: Option<String>,
    /// The description of the collection
    description: Option<String>,
    /// The symbol/ticker of the collection
    symbol: Option<String>,
    /// The URL of the collection website
    url: Option<String>,
    #[serde(default)]
    /// The addresses of creators
    creators: Vec<Address>,
    /// Collection tags
    tags: Option<Tags>,
    /// Collection-level supply
    #[serde(default)]
    supply: Supply,
    /// Collection royalties
    royalties: Option<RoyaltyPolicy>,
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

    pub fn write_move_init(&self, type_name: &str) -> String {
        let mut domains_str = String::new();

        domains_str.push_str(self.write_move_creators().as_str());
        domains_str.push_str(
            self.write_move_collection_display_info()
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(
            self.write_move_collection_symbol()
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(
            self.write_move_collection_url()
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(&self.supply().write_move_domain());
        domains_str.push_str(
            self.royalties
                .as_ref()
                .map(|royalties| royalties.write_move_init())
                .unwrap_or_default()
                .as_str(),
        );

        // Opt for `collection::create` over `collection::create_from_otw` in
        // order to statically assert `DelegatedWitness` gets created for the
        // `Collection<T>` type `T`.
        format!("

        let collection = nft_protocol::collection::create<{type_name}>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);{domains_str}

        sui::transfer::public_share_object(collection);"
        )
    }

    fn write_move_collection_display_info(&self) -> Option<String> {
        self.name().map(|name| {
            let description = self.description().unwrap_or_default();

            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::display_info::new(
                std::string::utf8(b\"{name}\"),
                std::string::utf8(b\"{description}\"),
            ),
        );"
            )
        })
    }

    fn write_move_collection_url(&self) -> Option<String> {
        self.url().as_ref().map(|url| {
            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            sui::url::new_unsafe_from_bytes(b\"{url}\"),
        );"
            )
        })
    }

    fn write_move_collection_symbol(&self) -> Option<String> {
        self.symbol().as_ref().map(|symbol| {
            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::symbol::new(std::string::utf8(b\"{symbol}\")),
        );",
            )
        })
    }

    // TODO: Separate out into `creators` module
    fn write_move_creators(&self) -> String {
        let mut code = String::new();

        if !self.creators.is_empty() {
            code.push_str(
                "

        let creators = sui::vec_set::empty();",
            );
            for address in self.creators.iter() {
                code.push_str(&format!(
                    "
        sui::vec_set::insert(&mut creators, @{address});"
                ));
            }

            code.push_str(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::new(creators),
        );",
            );
        };

        code
    }
}
