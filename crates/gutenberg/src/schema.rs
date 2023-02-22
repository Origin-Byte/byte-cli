//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::consts::DEFAULT_ADDRESS;
use crate::contract::modules::Imports;
use crate::err::GutenError;
use crate::models::settings::Settings;
use crate::models::{collection::CollectionData, nft::NftData};
use crate::storage::*;

use serde::{Deserialize, Serialize};
use strfmt::strfmt;

use std::collections::HashMap;

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Schema {
    pub collection: CollectionData,
    pub nft: NftData,
    pub settings: Settings,
    /// Creates a new marketplace with the collection
    // pub marketplace: Option<Marketplace>,
    pub contract: Option<String>,
    pub storage: Option<Storage>,
}

impl Schema {
    pub fn new(
        collection: CollectionData,
        nft: NftData,
        settings: Settings,
        contract: Option<String>,
        storage: Option<Storage>,
    ) -> Schema {
        Schema {
            collection,
            nft,
            settings,
            contract,
            storage,
        }
    }

    pub fn module_name(&self) -> String {
        self.collection.name.to_lowercase().replace(' ', "_")
    }

    pub fn witness_name(&self) -> String {
        self.collection.name.to_uppercase().replace(' ', "")
    }

    pub fn write_init_fn(&self) -> String {
        let signature = format!(
            "    fun init(witness: {}, ctx: &mut TxContext)",
            self.witness_name()
        );

        let init_collection = "let (mint_cap, collection) = collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness(&Witness {});\n";

        let domains = self.collection.write_domains();

        let feature_domains =
            self.settings.write_feature_domains(&self.collection);

        // let init_listings = self.settings.write_init_listings();

        let default_address = DEFAULT_ADDRESS.to_string();

        let first_creator = self
            .collection
            .creators
            .first()
            .unwrap_or_else(|| &default_address);

        let transfer_fns = self.settings.write_transfer_fns(first_creator);

        format!(
            "{signature} {{
        {init_collection}
        {domains}
        {feature_domains}
        {transfer_fns}
    }}",
            signature = signature,
            init_collection = init_collection,
            domains = domains,
            feature_domains = feature_domains,
            // init_listings = init_listings,
            transfer_fns = transfer_fns,
        )
    }

    pub fn write_entry_fns(&self) -> String {
        let mut code = String::new();

        if let Some(royalties) = &self.settings.royalties {
            let royalties_fn = royalties.write_entry_fn(&self.witness_name());
            code.push_str(royalties_fn.as_str());
        }

        let mint_fns = self
            .settings
            .mint_policies
            .write_mint_fns(&self.witness_name(), &self.nft);

        code.push_str(mint_fns.as_str());

        code
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
        let witness = self.witness_name();

        let imports = Imports::from_schema(self).write_imports(&self);

        let type_declarations = self.settings.write_type_declarations();

        let init_fn = self.write_init_fn();

        let entry_fns = self.write_entry_fns();

        // let init_marketplace = self
        //     .marketplace
        //     .as_ref()
        //     .map(Marketplace::init)
        //     .unwrap_or_else(String::new);

        // // Collate list of objects that need to be shared
        // // TODO: Use Marketplace::init and Listing::init functions to avoid
        // // explicit share
        // let share_marketplace = self
        //     .marketplace
        //     .as_ref()
        //     .map(Marketplace::share)
        //     .unwrap_or_default();

        // module {module_name}::{module_name} {{
        //     {imports}

        //     /// One time witness is only instantiated in the init method
        //     struct {witness} has drop {{}}

        //     /// Can be used for authorization of other actions post-creation. It is
        //     /// vital that this struct is not freely given to any contract, because it
        //     /// serves as an auth token.
        //     struct Witness has drop {{}}

        //     {type_declarations}

        //     {init_function}

        //     {entry_functions}
        // }}

        let mut vars = HashMap::<&'static str, &str>::new();

        vars.insert("module_name", &module_name);
        vars.insert("imports", &imports);
        vars.insert("witness", &witness);
        vars.insert("type_declarations", &type_declarations);
        vars.insert("init_function", &init_fn);
        vars.insert("entry_functions", &entry_fns);

        // Marketplace and Listing objects
        // vars.insert("init_marketplace", &init_marketplace);
        // vars.insert("share_marketplace", &share_marketplace);

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
