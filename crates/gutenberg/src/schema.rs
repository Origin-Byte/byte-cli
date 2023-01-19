//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::models::{
    collection::Collection,
    marketplace::{Listing, Marketplace},
    nft::Nft,
    royalties::Royalties,
};
use crate::GutenError;

use serde::{Deserialize, Serialize};
use strfmt::strfmt;

use std::collections::HashMap;

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Schema {
    pub collection: Collection,
    pub nft: Nft,
    pub royalties: Royalties,
    /// Creates a new marketplace with the collection
    pub marketplace: Option<Marketplace>,
    pub listings: Option<Vec<Listing>>,
}

impl Schema {
    pub fn new() -> Schema {
        Schema {
            collection: Collection::new(),
            nft: Nft::new(),
            royalties: Royalties::None,
            marketplace: Option::None,
            listings: Option::None,
        }
    }

    pub fn add_listing(&mut self, listing: Listing) {
        if let Some(listings) = self.listings.as_mut() {
            listings.push(listing)
        }
    }

    pub fn module_name(&self) -> String {
        self.collection.name.to_lowercase().replace(' ', "_")
    }

    /// Higher level method responsible for generating Move code from the
    /// struct `Schema` and dump it into a default folder
    /// `../sources/examples/<module_name>.move` or custom folder defined by
    /// the caller.
    pub fn write_move<W: std::io::Write>(
        &self,
        mut output: W,
    ) -> Result<(), GutenError> {
        let module_name = self.module_name();
        let witness = self.collection.name.to_uppercase().replace(' ', "");

        let init_marketplace = self
            .marketplace
            .as_ref()
            .map(Marketplace::init)
            .unwrap_or_else(String::new);

        let init_listings = self
            .listings
            .iter()
            .flatten()
            .map(Listing::init)
            .collect::<Vec<_>>();
        let init_listings = init_listings.join("\n    ");

        // Collate list of objects that need to be shared
        // TODO: Use Marketplace::init and Listing::init functions to avoid
        // explicit share
        let share_marketplace = self
            .marketplace
            .as_ref()
            .map(Marketplace::share)
            .unwrap_or_default();

        let mut vars = HashMap::<&'static str, &str>::new();

        let tags = self.collection.tags.init();
        let royalty_strategy = self.royalties.write();
        let mint_functions = self.nft.mint_fns(&witness);
        let url = self
            .collection
            .url
            .as_ref()
            .map(String::as_str)
            .unwrap_or_default();

        vars.insert("module_name", &module_name);
        vars.insert("witness", &witness);
        vars.insert("name", &self.collection.name);
        vars.insert("description", &self.collection.description);
        vars.insert("symbol", &self.collection.symbol);
        vars.insert("royalty_strategy", &royalty_strategy);
        vars.insert("mint_functions", &mint_functions);
        vars.insert("tags", &tags);
        vars.insert("url", &url);

        // Marketplace and Listing objects
        vars.insert("init_marketplace", &init_marketplace);
        vars.insert("init_listings", &init_listings);
        vars.insert("share_marketplace", &share_marketplace);

        let vars: HashMap<String, String> = vars
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        output.write_all(
            strfmt(include_str!("../templates/template.move"), &vars)
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
}
