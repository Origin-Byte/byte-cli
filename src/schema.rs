//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::err::GutenError;
use crate::types::{Launchpad, NftType, Slot, Tag};

use serde::Deserialize;
use strfmt::strfmt;

use std::collections::HashMap;
use std::fmt::Write;
use std::fs;

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Schema {
    pub collection: Collection,
    pub nft_type: NftType,
    /// Creates a new launchpad with the collection
    pub launchpad: Option<Launchpad>,
    /// Registers slots with a provided launchpad or the launchpad created by
    /// the collection
    pub slots: Option<Vec<Slot>>,
}

/// Contains the metadata fields of the collection
#[derive(Debug, Deserialize)]
pub struct Collection {
    /// The name of the collection
    pub name: Box<str>,
    /// The description of the collection
    pub description: Box<str>,
    /// The symbol/ticker of the collection
    pub symbol: Box<str>,
    /// A set of strings that categorize the domain in which the NFT operates
    pub tags: Vec<Tag>,
    /// The royalty fees creators accumulate on the sale of NFTs
    pub royalty_fee_bps: Box<str>,
    /// Field for extra data
    pub url: Box<str>,
}

impl Schema {
    pub fn module_name(&self) -> Box<str> {
        self.collection
            .name
            .to_lowercase()
            .replace(' ', "_")
            .into_boxed_str()
    }

    /// Higher level method responsible for generating Move code from the
    /// struct `Schema` and dump it into a default folder
    /// `../sources/examples/<module_name>.move` or custom folder defined by
    /// the caller.
    pub fn write_move<W: std::io::Write>(
        &self,
        mut output: W,
    ) -> Result<(), GutenError> {
        let file_path = "templates/template.move";
        let fmt = fs::read_to_string(file_path)
            .expect("Should have been able to read the file");

        let module_name = self.module_name();

        let witness = self
            .collection
            .name
            .to_uppercase()
            .replace(' ', "")
            .into_boxed_str();

        let tags = self.write_tags();

        if self.slots.is_some() && self.launchpad.is_none() {
            return Err(GutenError::SlotsMustInitializeLaunchpad);
        }

        let init_launchpad = self
            .launchpad
            .as_ref()
            .map(Launchpad::init)
            .unwrap_or_else(String::new)
            .into_boxed_str();

        let init_slots = self
            .slots
            .iter()
            .flatten()
            .map(Slot::init)
            .collect::<Vec<_>>();
        let init_slots = init_slots.join("\n    ").into_boxed_str();

        // Collate list of objects that need to be shared
        // TODO: Use Launchpad::init and Slot::init functions to avoid explicit share
        let share_launchpad = self
            .launchpad
            .as_ref()
            .map(Launchpad::share)
            .unwrap_or_default()
            .into_boxed_str();

        let mut vars = HashMap::new();

        vars.insert("module_name", &module_name);
        vars.insert("witness", &witness);
        vars.insert("name", &self.collection.name);
        vars.insert("description", &self.collection.description);
        vars.insert("url", &self.collection.url);
        vars.insert("symbol", &self.collection.symbol);
        vars.insert("royalty_fee_bps", &self.collection.royalty_fee_bps);
        vars.insert("tags", &tags);

        // Launchpad and slot objects
        vars.insert("init_launchpad", &init_launchpad);
        vars.insert("init_slots", &init_slots);
        vars.insert("share_launchpad", &share_launchpad);

        let vars: HashMap<String, String> = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        output.write_all(
            strfmt(&fmt, &vars)
                // This is expected not to result in an error since we
                // have explicitly handled all error cases
                .unwrap_or_else(|err| {
                    panic!(
                        "This error is not expected and should not occur: {}",
                        err
                    )
                })
                .as_bytes(),
        )?;

        Ok(())
    }

    /// Generates Move code to push tags to a Move `vector` structure
    pub fn write_tags(&self) -> Box<str> {
        let mut out = String::from("let tags = tags::empty(ctx);\n");

        for tag in self.collection.tags.iter() {
            out.write_fmt(format_args!(
                "        tags::add_tag(&mut tags, tags::{}());\n",
                tag.to_string()
            ))
            .unwrap();
        }

        out.push_str(
            "        tags::add_collection_tag_domain(&mut collection, &mut mint_cap, tags);"
        );

        out.into_boxed_str()
    }
}
