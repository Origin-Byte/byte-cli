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

use crate::contract::modules::sui::Display;

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// The named address that the module is published under
    module_alias: Option<String>,
    pub collection: CollectionData,
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

        let tags = self.settings.write_tags();
        let display = Display::write_display(&self.nft.type_name);
        let request_policies = self
            .settings
            .request_policies
            .write_policies(&self.nft.type_name);

        let allowlist = format!("
        // Setup Allowlist
        let (allowlist, allowlist_cap) = ob_allowlist::allowlist::new(ctx);

        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::orderbook::Witness>(
            &allowlist_cap, &mut allowlist,
        );
        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::bidding::Witness>(
            &allowlist_cap, &mut allowlist,
        );"          
        );

        format!(
            "    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let sender = sui::tx_context::sender(ctx);

        let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<{witness}, {type_name}>(
            &witness, option::none(), ctx
        );

        // Init Publisher
        let publisher = sui::package::claim(witness, ctx);

        // Init Tags
        {tags}

        // Init Display
        {display}

        let delegated_witness = nft_protocol::witness::from_witness(Witness {{}});
{domains}{feature_domains}{request_policies}{allowlist}{transfer_fns}    }}",
            witness = self.witness_name(),
            type_name = self.nft.type_name
        )
    }

    pub fn write_entry_fns(&self) -> String {
        let mut code = String::new();

        let mint_fns =
            self.settings.mint_policies.write_mint_fns(&self.collection);

        code.push_str(mint_fns.as_str());

        code
    }

    pub fn write_tests(&self) -> String {
        let type_name = &self.nft.type_name;
        let witness = self.witness_name();
        format!(
            "
    #[test_only]
    use sui::test_scenario::{{Self, ctx}};
    #[test_only]
    use nft_protocol::collection::Collection;

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {{
        let scenario = test_scenario::begin(CREATOR);

        init({witness} {{}}, ctx(&mut scenario));
        test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(test_scenario::has_most_recent_shared<Collection<{type_name}>>(), 0);

        let mint_cap = test_scenario::take_from_address<MintCap<{type_name}>>(
            &scenario, CREATOR,
        );

        test_scenario::return_to_address(CREATOR, mint_cap);
        test_scenario::next_tx(&mut scenario, CREATOR);

        test_scenario::end(scenario);
    }}

    #[test]
    fun it_mints_nft() {{
        let scenario = test_scenario::begin(CREATOR);
        init({witness} {{}}, ctx(&mut scenario));

        test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = test_scenario::take_from_address<MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );

        let warehouse = warehouse::new<{type_name}>(ctx(&mut scenario));

        mint_nft(
            string::utf8(b\"TEST NAME\"),
            string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[ascii::string(b\"avg_return\")],
            vector[ascii::string(b\"24%\")],
            &mut mint_cap,
            &mut warehouse,
            ctx(&mut scenario)
        );

        transfer::public_transfer(warehouse, CREATOR);
        test_scenario::return_to_address(CREATOR, mint_cap);
        test_scenario::end(scenario);
    }}")
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

        let nft_struct = self.nft.write_struct();

        let mut vars = HashMap::<&'static str, &str>::new();

        let tests = self.write_tests();

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
        vars.insert("nft_struct", &nft_struct);
        vars.insert("tests", &tests);

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
