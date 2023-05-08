//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
pub mod supply;

use serde::{Deserialize, Serialize};

use crate::{
    consts::{MAX_CREATORS_LENGTH, MAX_SYMBOL_LENGTH},
    contract::modules::DisplayInfoMod,
    err::GutenError,
    utils::validate_address,
};

use supply::SupplyPolicy;

/// Contains the metadata fields of the collection
#[derive(Debug, Serialize, Deserialize, Default)]
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
    pub creators: Vec<String>,
    #[serde(default)]
    pub supply_policy: SupplyPolicy,
}

impl CollectionData {
    pub fn new(
        name: String,
        description: Option<String>,
        symbol: Option<String>,
        url: Option<String>,
        creators: Vec<String>,
        supply_policy: SupplyPolicy,
    ) -> CollectionData {
        CollectionData {
            name,
            description,
            symbol,
            url,
            creators,
            supply_policy,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.name.is_empty()
            && self.description.is_none()
            && self.symbol.is_none()
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
        self.description = Some(description);
    }

    pub fn set_symbol(&mut self, mut symbol: String) -> Result<(), GutenError> {
        if !symbol.chars().all(|c| c.is_ascii_alphanumeric()) {
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

    pub fn set_supply_policy(&mut self, supply_policy: SupplyPolicy) {
        self.supply_policy = supply_policy;
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
                let address = if address == "sui::tx_context::sender(ctx)" {
                    address.clone()
                } else {
                    format!("@{address}")
                };

                code.push_str(&format!(
                    "        sui::vec_set::insert(&mut creators, {address});\n"
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
}
