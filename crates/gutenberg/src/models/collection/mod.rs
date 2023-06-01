//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
pub mod tags;

use serde::{Deserialize, Serialize};

use crate::{contract::modules::DisplayInfoMod, err::GutenError};

use self::tags::Tags;
use super::Address;

/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CollectionData {
    /// The name of the collection
    pub name: String,
    /// The description of the collection
    pub description: Option<String>,
    /// The symbol/ticker of the collection
    pub symbol: Option<String>,
    /// The URL of the collection website
    pub url: Option<String>,
    #[serde(default)]
    /// The addresses of creators
    pub creators: Vec<Address>,
    pub tags: Option<Tags>,
}

impl CollectionData {
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
            .map(Address::new)
            .collect::<Result<Vec<Address>, GutenError>>()?;

        // Validate that creator strings are addresses
        self.creators = creator_addresses;

        Ok(())
    }

    pub fn write_domains(&self) -> String {
        let mut code = String::new();

        code.push_str(self.write_creators().as_str());

        if let Some(display) = DisplayInfoMod::add_collection_display_info(self)
        {
            code.push_str(&display);
        }

        if let Some(symbol) = DisplayInfoMod::add_collection_symbol(self) {
            code.push_str(&symbol);
        }

        if let Some(url) = DisplayInfoMod::add_collection_url(self) {
            code.push_str(&url);
        }

        code
    }

    pub fn write_creators(&self) -> String {
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
        let creators = sui::vec_set::empty();\n",
            );
            for address in self.creators.iter() {
                code.push_str(&format!(
                    "        sui::vec_set::insert(&mut creators, @{address});\n"
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
        );\n",
            creators_domain
        ));

        code
    }

    pub fn write_tags(&self) -> String {
        self.tags
            .as_ref()
            .map(|tags| tags.write_move())
            .unwrap_or_default()
    }
}

/// De-unicodes and removes all unknown characters
fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}
