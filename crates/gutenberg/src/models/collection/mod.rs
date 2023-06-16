//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
#[cfg(feature = "full")]
#[macro_use]
mod full {
    pub mod royalties;
    pub mod supply;

    pub use royalties::{RoyaltyPolicy, Share};
    pub use supply::Supply;
}
mod tags;

use super::{nft::NftData, Address};
#[cfg(feature = "full")]
pub use full::*;
pub use tags::{Tag, Tags};

use serde::{Deserialize, Serialize};

#[cfg(feature = "full")]
/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    /// The name of the collection
    name: String,
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
    supply: Supply,
    /// Collection royalties
    royalties: Option<RoyaltyPolicy>,
}

#[cfg(not(feature = "full"))]
/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    /// The name of the collection
    name: String,
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
}

#[cfg(feature = "full")]
impl CollectionData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
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

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn royalties(&self) -> &Option<RoyaltyPolicy> {
        &self.royalties
    }
}

#[cfg(not(feature = "full"))]
impl CollectionData {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: String,
        description: Option<String>,
        symbol: Option<String>,
        url: Option<String>,
        creators: Vec<Address>,
        tags: Option<Tags>,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
            tags,
        }
    }
}

impl CollectionData {
    pub fn name(&self) -> String {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        deunicode(&self.name)
    }

    pub fn description(&self) -> Option<String> {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.description
            .as_ref()
            .map(|description| deunicode(description))
    }

    // Retains only alphanumeric characters
    fn escaped_name(&self) -> String {
        self.name()
            .chars()
            .filter_map(|char| match char {
                '-' => Some('_'),
                ' ' => Some('_'),
                char => char.is_ascii_alphanumeric().then_some(char),
            })
            .collect()
    }

    pub fn package_name(&self) -> String {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.escaped_name().to_lowercase()
    }

    pub fn witness_name(&self) -> String {
        // Since `CollectionData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.escaped_name().to_uppercase()
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

    pub fn creators(&self) -> &Vec<Address> {
        &self.creators
    }

    pub fn tags(&self) -> &Option<Tags> {
        &self.tags
    }

    /// Whether collection has royalty policy defined
    pub fn has_royalties(&self) -> bool {
        #[cfg(feature = "full")]
        let has_royalties = self.royalties().is_some();
        #[cfg(not(feature = "full"))]
        let has_royalties = false;

        has_royalties
    }

    /// Whether `&mut Collection` needs to be passed into mint methods
    pub fn requires_collection(&self) -> bool {
        #[cfg(feature = "full")]
        let requires_collection = self.supply().requires_collection();
        #[cfg(not(feature = "full"))]
        let requires_collection = false;

        requires_collection
    }

    pub fn write_move_init(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut domains_str = String::new();
        domains_str.push_str(&self.write_move_creators());
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
        #[cfg(feature = "full")]
        domains_str.push_str(&self.supply().write_move_domain());
        #[cfg(feature = "full")]
        domains_str.push_str(
            self.royalties
                .as_ref()
                .map(|royalties| royalties.write_move_init())
                .unwrap_or_default()
                .as_str(),
        );
        domains_str.push_str(
            self.tags
                .as_ref()
                .map(|tags| tags.write_move_init())
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
        let name = self.name();
        self.description().as_ref().map(|description| {
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

        let creators_domain = if self.creators.is_empty() {
            format!(
                "nft_protocol::creators::from_address<{witness_name}, Witness>(
                &Witness {{}}, sui::tx_context::sender(ctx),
            )",
                witness_name = self.witness_name(),
            )
        } else {
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

            "nft_protocol::creators::new(creators)".to_string()
        };

        code.push_str(&format!(
            "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            {},
        );",
            creators_domain
        ));

        code
    }
}

/// De-unicodes and removes all unknown characters
fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}
