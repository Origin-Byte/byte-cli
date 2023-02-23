//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

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
    pub description: String,
    /// The symbol/ticker of the collection
    pub symbol: String,
    /// The URL of the collection website
    pub url: Option<String>,
    /// The addresses of creators
    pub creators: BTreeSet<String>,
}

impl CollectionData {
    pub fn new(
        name: String,
        description: String,
        symbol: String,
        url: Option<String>,
        creators: BTreeSet<String>,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
            && self.description.is_empty()
            && self.symbol.is_empty()
            && self.url.is_none()
            && self.creators.is_empty()
    }

    pub fn witness_name(&self) -> String {
        self.name.to_uppercase().replace(' ', "")
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
        self.description = description;
    }

    pub fn set_symbol(&mut self, mut symbol: String) -> Result<(), GutenError> {
        if symbol.chars().all(|c| matches!(c, 'a'..='z')) {
            return Err(GutenError::UnsupportedCollectionInput(format!(
                "The collection symbol provided `{}` should only have alphanumeric characters.",
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
        self.symbol = symbol;

        Ok(())
    }

    pub fn set_url(&mut self, url_string: String) -> Result<(), GutenError> {
        // Just here for validation
        // TODO: Add back this
        let mut url: String;

        if url_string.starts_with("www.") {
            url = String::from("http://");
            url.push_str(url_string.split_at(4).0);
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
        creators: BTreeSet<String>,
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

        code.push_str(DisplayMod::add_collection_display(self).as_str());
        code.push_str(DisplayMod::add_collection_symbol(self).as_str());

        if let Some(_url) = &self.url {
            code.push_str(DisplayMod::add_collection_url(self).as_str());
        }

        code
    }

    pub fn write_creators(&self) -> String {
        let mut code = String::new();

        let creators_domain = if self.creators.len() == 1 {
            format!(
                "
    creators::from_address<{name}, Witness>(
        &Witness {{}}, {address}, ctx,
    )",
                name = self.name,
                address = self.creators.first().unwrap(),
            )
        } else {
            code.push_str("let creators = vec_set::empty();\n");

            self.creators
                .iter()
                .map(|addr| {
                    code.push_str(
                        format!(
                            "        vec_set::insert(&mut creators, @{});\n",
                            addr
                        )
                        .as_str(),
                    );
                })
                .for_each(drop);

            format!(
                "creators::from_creators<{witness}, Witness>(
                &Witness {{}}, creators,
            )",
                witness = self.witness_name()
            )
        };

        let add_domain = format!(
            "
        collection::add_domain(
            delegated_witness,
            &mut collection,
            {},
        );\n",
            creators_domain
        );

        code.push_str(add_domain.as_str());

        code
    }
}
