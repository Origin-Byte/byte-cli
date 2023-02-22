//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use serde::{Deserialize, Serialize};

use crate::{
    consts::{MAX_CREATORS_LENGTH, MAX_SYMBOL_LENGTH},
    contract::modules::DisplayMod,
    err::GutenError,
    utils::validate_address,
};

/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Default)]
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
    pub creators: Vec<String>,
}

impl CollectionData {
    pub fn new(
        name: String,
        description: Option<String>,
        symbol: Option<String>,
        url: Option<String>,
        creators: Vec<String>,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
        }
    }

    pub fn witness_name(&self) -> String {
        self.name.to_uppercase().replace(' ', "")
    }

    pub fn set_name(&mut self, mut name: String) -> Result<(), GutenError> {
        if !name.chars().all(|c| matches!(c, 'a'..='z')) {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection name provided `{}` should not have alphanumeric characters.",
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
        if !symbol.chars().all(|c| matches!(c, 'a'..='z')) {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection symbol provided `{}` should not have alphanumeric characters.",
                symbol
            )));
        }

        if symbol.len() > MAX_SYMBOL_LENGTH {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection symbol `{}` has {} characters, which is above the maximum length of {}.",
                symbol,
                symbol.len(),
                MAX_SYMBOL_LENGTH
            )));
        }

        symbol = symbol.to_uppercase();
        self.symbol = Some(symbol);

        Ok(())
    }

    pub fn set_url(&mut self, url_string: String) -> Result<(), GutenError> {
        // Just here for validation
        let _ = url::Url::parse(&url_string).map_err(|err| {
            GutenError::UnsupportedCollectionInput(format!(
                "The following error has occured: {}
The Collection URL input `{}` is not valid.",
                err, url_string
            ))
        })?;

        self.url = Some(url_string);

        Ok(())
    }

    pub fn set_creators(
        &mut self,
        creators: Vec<String>,
    ) -> Result<(), GutenError> {
        if creators.len() > MAX_CREATORS_LENGTH {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The creators list provided surpasses the limit of {}. The list provided has {} addresses.",
                MAX_CREATORS_LENGTH, creators.len()
            )));
        }

        // Guarantees that creator addresses are valid
        creators
            .iter()
            .map(|creator| validate_address(creator))
            .collect::<Result<(), GutenError>>()?;

        // Validate that creator strings are addresses
        self.creators = creators;

        Ok(())
    }

    pub fn write_domains(&self) -> String {
        let mut code = String::new();

        code.push_str(self.write_creators().as_str());

        if let Some(display) = DisplayMod::add_collection_display(self) {
            code.push_str(&display);
        }

        if let Some(symbol) = DisplayMod::add_collection_symbol(self) {
            code.push_str(&symbol);
        }

        if let Some(url) = DisplayMod::add_collection_url(self) {
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
            for addr in self.creators.iter() {
                code.push_str(&format!(
                    "        sui::vec_set::insert(&mut creators, @{addr});\n"
                ));
            }

            format!(
                "creators::from_creators<{witness}, Witness>(
                &Witness {{}}, creators,
            )",
                witness = self.witness_name()
            )
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
}
