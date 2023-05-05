//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::err::GutenError;
use crate::models::launchpad::Launchpad;
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
    /// The named address that the module is published under
    module_alias: Option<String>,
    pub collection: CollectionData,
    #[serde(default)]
    pub nft: NftData,
    #[serde(default)]
    pub settings: Settings,
    pub launchpad: Option<Launchpad>,
    pub contract: Option<String>,
    pub storage: Option<Storage>,
}

impl Schema {
    pub fn new(
        collection: CollectionData,
        nft: NftData,
        settings: Settings,
        launchpad: Option<Launchpad>,
        contract: Option<String>,
        storage: Option<Storage>,
    ) -> Schema {
        Schema {
            module_alias: None,
            collection,
            nft,
            settings,
            launchpad,
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
        let domains = self.collection.write_domains();

        let feature_domains =
            self.settings.write_feature_domains(&self.collection);

        let transfer_fns = self
            .settings
            .write_transfer_fns(self.collection.creators.first());

        format!(
            "    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let (mint_cap, collection) = nft_protocol::collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness<{witness}, Witness>(&Witness {{}});
{domains}{feature_domains}{transfer_fns}    }}",
            witness = self.witness_name()
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
            .write_mint_fns(&self.collection, &self.nft);

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

        if let Some(module_alias) = &self.module_alias {
            vars.insert("module_alias", module_alias);
        } else {
            vars.insert("module_alias", &module_name);
        }

        vars.insert("module_name", &module_name);
        vars.insert("witness", &witness);
        vars.insert("type_declarations", &type_declarations);
        vars.insert("init_function", &init_fn);
        vars.insert("entry_functions", &entry_fns);
        vars.insert(
            "imports",
            "
    use std::ascii;
    use std::option;
    use std::string::{Self, String};

    use sui::url::{Self, Url};
    use sui::sui::SUI;
    use sui::display;
    use sui::transfer;
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};

    use nft_protocol::mint_cap;
    use nft_protocol::mint_event;
    use nft_protocol::creators;
    use nft_protocol::attributes::{Self, Attributes};
    use nft_protocol::collection;
    use nft_protocol::display_info;
    use nft_protocol::mint_cap::MintCap;
    use nft_protocol::royalty;
    use nft_protocol::royalty_strategy_bps;
    use nft_protocol::tags;
    use nft_protocol::transfer_allowlist;

    use ob_utils::utils;
    use ob_utils::display as ob_display;

    use ob_permissions::witness;
    use ob_request::transfer_request;
    use ob_launchpad::warehouse::{Self, Warehouse};

    use ob_allowlist::allowlist;

    use liquidity_layer_v1::orderbook;
    use liquidity_layer_v1::bidding;",
        );

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
