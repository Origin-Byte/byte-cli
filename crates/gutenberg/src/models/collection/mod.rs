//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
mod mint_cap;
mod orderbook;
mod request;
mod royalties;
mod supply;
mod tags;

use self::{
    mint_cap::MintCap, orderbook::Orderbook, request::RequestPolicies,
    royalties::RoyaltyPolicy, supply::Supply, tags::Tags,
};
use super::{nft::NftData, Address};
use crate::err::GutenError;
use serde::{Deserialize, Serialize};

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
    supply: Supply,
    mint_cap: MintCap,
    royalties: Option<RoyaltyPolicy>,
    tags: Option<Tags>,
    #[serde(default)]
    request_policies: RequestPolicies,
    #[serde(default)]
    orderbook: Orderbook,
}

impl Default for CollectionData {
    /// TODO: `CollectionData` should not implement `Default` as there isn't a notion
    /// of a default collection.
    ///
    /// This implementation provides a reasonable default that shouldn't break
    /// anything.
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            symbol: None,
            url: Some("https://originbyte.io".to_string()),
            supply: Supply::Untracked,
            mint_cap: MintCap::new(None),
            creators: Vec::new(),
            royalties: None,
            tags: None,
            request_policies: RequestPolicies::default(),
            orderbook: Orderbook::default(),
        }
    }
}

impl CollectionData {
    pub fn new(
        name: String,
        description: Option<String>,
        symbol: Option<String>,
        url: Option<String>,
        creators: Vec<Address>,
        supply: Supply,
        mint_cap: MintCap,
        royalties: Option<RoyaltyPolicy>,
        tags: Option<Tags>,
        request_policies: RequestPolicies,
        orderbook: Orderbook,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
            supply,
            mint_cap,
            royalties,
            tags,
            request_policies,
            orderbook,
        }
    }

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

    pub fn supply(&self) -> &Supply {
        &self.supply
    }

    pub fn request_policies(&self) -> &RequestPolicies {
        &self.request_policies
    }

    pub fn set_name(&mut self, mut name: String) -> Result<(), GutenError> {
        if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection name provided `{}` should only have alphanumeric characters.",
                name
            )));
        }

        name = name.to_lowercase();
        self.name = name;

        Ok(())
    }

    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    pub fn set_symbol(&mut self, mut symbol: String) -> Result<(), GutenError> {
        if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection symbol provided `{}` should only have alphanumeric characters.",
                symbol
            )));
        }

        symbol = symbol.to_uppercase();
        self.symbol = Some(symbol);

        Ok(())
    }

    pub fn set_url(&mut self, url_string: String) -> Result<(), GutenError> {
        let mut url: String;

        if url_string.starts_with("www.") {
            url = String::from("http://");
            url.push_str(url_string.split_at(4).1);
        } else {
            url = url_string;
        }

        let _ = url::Url::parse(&url).map_err(|err| {
            GutenError::UnsupportedCollectionInput(format!(
                "The following error has occured: {}
        The Collection URL input `{}` is not valid.",
                err, url
            ))
        })?;

        self.url = Some(url);

        Ok(())
    }

    pub fn set_creators(
        &mut self,
        creators: Vec<String>,
    ) -> Result<(), GutenError> {
        // Guarantees that creator addresses are valid
        let creator_addresses = creators
            .into_iter()
            .map(|creator| Address::new(creator))
            .collect::<Result<Vec<Address>, GutenError>>()?;

        // Validate that creator strings are addresses
        self.creators = creator_addresses;

        Ok(())
    }

    pub fn write_move_init(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut domains_str = String::new();
        domains_str.push_str(
            &self
                .mint_cap
                .write_move_init(&self.witness_name(), type_name),
        );
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
        domains_str.push_str(&self.supply().write_move_domain());
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
        domains_str.push_str(
            "

        let publisher = sui::package::claim(witness, ctx);",
        );
        domains_str.push_str(&self.request_policies.write_move_init(nft_data));
        domains_str.push_str(&self.orderbook.write_move_init(type_name));

        format!("

        let collection = nft_protocol::collection::create<{type_name}>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);{domains_str}

        sui::transfer::public_share_object(collection);"
        )
    }

    pub fn write_move_defs(&self, type_name: &str) -> String {
        self.orderbook.write_move_defs(type_name)
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

    pub fn write_move_collection_url(&self) -> Option<String> {
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

    pub fn write_move_collection_symbol(&self) -> Option<String> {
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
