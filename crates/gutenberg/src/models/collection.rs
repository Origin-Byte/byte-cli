//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use serde::{Deserialize, Serialize};

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
    pub creators: Vec<String>,
}

impl CollectionData {
    pub fn new(
        name: String,
        description: String,
        symbol: String,
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

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn set_description(&mut self, description: String) {
        self.description = description;
    }

    pub fn set_symbol(&mut self, symbol: String) {
        self.symbol = symbol;
    }

    pub fn set_url(&mut self, url: String) {
        self.url = Some(url);
    }

    pub fn set_creators(&mut self, creators: Vec<String>) {
        self.creators = creators;
    }

    pub fn write_domains(&self) -> String {
        let code = String::new();

        code.push_str(self.write_creators().as_str());

        code.push_str(self.write_display().as_str());
        code.push_str(self.write_symbol().as_str());

        if self.url.is_some() {
            code.push_str(self.write_url().as_str());
        }

        code
    }

    pub fn write_display(&self) -> String {
        format!(
            "
            display::add_collection_display_domain(
                delegated_witness,
                &mut collection,
                string::utf8(b\"{name}\"),
                string::utf8(b\"{description}
            );\n",
            name = self.name,
            description = self.description,
        )
    }

    pub fn write_url(&self) -> String {
        format!(
            "
            display::add_collection_url_domain(
                delegated_witness,
                &mut collection,
                sui::url::new_unsafe_from_bytes(b\"{url}\"),
                ctx,
            );\n",
            url = self.url.expect("No collection url setup found"),
        )
    }

    pub fn write_symbol(&self) -> String {
        format!(
            "
            display::add_collection_symbol_domain(
                delegated_witness,
                &mut collection,
                string::utf8(b\"{symbol}\"),
                ctx,
            );",
            symbol = self.symbol,
        )
    }

    pub fn write_creators(&self) -> String {
        let code = String::new();

        let creators_domain = if self.creators.len() == 1 {
            format!(
                "
                creators::from_address<{name}, Witness>(
                    &Witness {{}}, {address}, ctx,
                )",
                name = self.name,
                address = self.creators[0],
            )
        } else {
            code.push_str("let creators = vec_set::empty();\n");

            self.creators
                .iter()
                .map(|addr| {
                    code.push_str(
                        format!("vec_set::insert(&mut creators, {});\n", addr)
                            .as_str(),
                    );
                })
                .collect::<()>();

            format!(
                "
                creators::from_creators<{name}, Witness>(
                    &Witness {{}}, creators, ctx
                )",
                name = self.name
            )
        };

        let add_domain = format!(
            "
            collection::add_domain(
                delegated_witness,
                &mut collection,
                {},
            );",
            creators_domain
        );

        code.push_str(add_domain.as_str());

        code
    }
}
