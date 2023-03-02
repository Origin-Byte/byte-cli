//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::err::GutenError;
use crate::models::launchpad::Launchpad;
use crate::models::settings::Settings;
use crate::models::{collection::CollectionData, nft::NftData};

use serde::{Deserialize, Serialize};
use strfmt::strfmt;

use std::collections::HashMap;

use crate::contract::modules::Display;

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// The named address that the module is published under
    package_name: Option<String>,
    pub collection: CollectionData,
    pub nft: NftData,
    pub settings: Settings,
    pub launchpad: Option<Launchpad>,
}

impl Schema {
    pub fn new(
        collection: CollectionData,
        nft: NftData,
        settings: Settings,
        launchpad: Option<Launchpad>,
    ) -> Schema {
        Schema {
            package_name: None,
            collection,
            nft,
            settings,
            launchpad,
        }
    }

    pub fn module_name(&self) -> String {
        self.collection.name.to_lowercase().replace(' ', "_")
    }

    pub fn witness_name(&self) -> String {
        self.module_name().to_uppercase()
    }

    pub fn write_init_fn(&self) -> Result<String, GutenError> {
        let domains = self.collection.write_domains();

        let feature_domains =
            self.settings.write_feature_domains(&self.collection)?;

        let transfer_fns = self.settings.write_transfer_fns();

        let tags = self.collection.write_tags();
        let display = Display::write_display(&self.nft.type_name);
        let request_policies = self
            .settings
            .request_policies
            .write_policies(&self.nft.type_name);

        let orderbook =
            self.settings.orderbook.write_orderbook(&self.nft.type_name);

        let witness = self.witness_name();
        let type_name = &self.nft.type_name;

        let create_collection = self
            .settings
            .mint_policies
            .write_collection_create_with_mint_cap(&witness, &type_name);

        format!(
            "    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        {create_collection}

        // Init Publisher
        let publisher = sui::package::claim(witness, ctx);

        // Init Tags
        {tags}

        // Init Display
        {display}

        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});
{domains}{feature_domains}{request_policies}{orderbook}{transfer_fns}    }}" 
        )
    }

    pub fn write_entry_fns(&self) -> String {
        let mut code = String::new();

        let mint_fns = self
            .settings
            .mint_policies
            .write_mint_fns(&self.nft.type_name);

        code.push_str(mint_fns.as_str());

        let orderbook_fns =
            self.settings.orderbook.write_entry_fns(&self.nft.type_name);

        code.push_str(orderbook_fns.as_str());

        if self.settings.burn {
            code.push_str(&self.settings.write_burn_fns(&self.nft.type_name));
        }

        code
    }

    pub fn write_tests(&self) -> String {
        let type_name = &self.nft.type_name;
        let witness = self.witness_name();
        format!(
            "
    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {{
        let scenario = sui::test_scenario::begin(CREATOR);

        init({witness} {{}}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(sui::test_scenario::has_most_recent_shared<nft_protocol::collection::Collection<{type_name}>>(), 0);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario, CREATOR,
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        sui::test_scenario::end(scenario);
    }}

    #[test]
    fun it_mints_nft() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );

        let warehouse = ob_launchpad::warehouse::new<{type_name}>(sui::test_scenario::ctx(&mut scenario));

        mint_nft(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::end(scenario);
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

        let init_fn = self.write_init_fn()?;

        let entry_fns = self.write_entry_fns();

        let nft_struct = self.nft.write_struct();

        let mut vars = HashMap::<&'static str, &str>::new();

        let tests = self.write_tests();

        if let Some(package_name) = &self.package_name {
            vars.insert("package_name", package_name);
        } else {
            vars.insert("package_name", &module_name);
        }
        vars.insert("module_name", &module_name);
        vars.insert("witness", &witness);
        vars.insert("type_declarations", &type_declarations);
        vars.insert("init_function", &init_fn);
        vars.insert("entry_functions", &entry_fns);
        vars.insert("nft_struct", &nft_struct);
        vars.insert("tests", &tests);

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
