//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use crate::contract::modules::Display;
use crate::err::GutenError;
use crate::models::settings::Settings;
use crate::models::{collection::CollectionData, nft::NftData};

use serde::{Deserialize, Serialize};

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize, Builder)]
#[builder(derive(Debug, Serialize, Deserialize))]
#[builder(setter(into))]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// The named address that the module is published under
    package_name: Option<String>,
    #[builder(field(public))]
    collection: CollectionData,
    #[builder(field(public))]
    nft: NftData,
    #[builder(field(public))]
    pub settings: Settings,
}

impl Schema {
    pub fn new(
        collection: CollectionData,
        nft: NftData,
        settings: Settings,
    ) -> Schema {
        Schema {
            package_name: None,
            collection,
            nft,
            settings,
        }
    }

    pub fn package_name(&self) -> String {
        match &self.package_name {
            Some(package_name) => package_name.clone(),
            None => self.collection().package_name(),
        }
    }

    pub fn collection(&self) -> &CollectionData {
        &self.collection
    }

    pub fn nft(&self) -> &NftData {
        &self.nft
    }

    pub fn write_init_fn(&self) -> String {
        let domains = self.collection.write_move_domains();

        let feature_domains =
            self.settings.write_feature_domains(&self.collection);

        let tags = self.collection.write_move_tags();

        let type_name = self.nft.type_name();
        let display = Display::write_display(type_name);

        let orderbook = self.settings.orderbook.write_orderbook(type_name);

        let witness = self.collection().witness_name();

        let create_collection = self
            .settings
            .mint_policies
            .write_collection_create_with_mint_cap(&witness, type_name);

        let request_policies = self.settings.write_request_policies(&self.nft);
        let transfer_fns = self.settings.write_transfer_fns(&self.nft);

        format!(
            "

    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        {create_collection}

        let publisher = sui::package::claim(witness, ctx);
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});
{tags}{display}{domains}{feature_domains}{request_policies}{orderbook}{transfer_fns}
    }}"
        )
    }

    pub fn write_entry_fns(&self) -> String {
        let mut code = String::new();

        let nft_data = &self.nft;
        let collection_data = &self.collection;

        code.push_str(
            &self
                .settings
                .mint_policies
                .write_move_mint_fns(nft_data, collection_data),
        );

        // TODO: Conditional on importing LiquidityLayer V1
        code.push_str(
            self.settings.orderbook.write_entry_fns(nft_data).as_str(),
        );

        code.push_str(&self.nft.write_dynamic_fns());
        code.push_str(&self.nft.write_burn_fns(collection_data));

        code
    }

    pub fn write_tests(&self) -> String {
        let type_name = self.nft.type_name();
        let collection = self.collection();

        let witness = collection.witness_name();
        let supply = collection.supply();

        let requires_collection = supply.requires_collection();
        let collection_take_str = requires_collection.then(|| format!("

        let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<{type_name}>>(
            &scenario,
        );")).unwrap_or_default();

        let collection_param_str = requires_collection
            .then_some(
                "
            &mut collection,",
            )
            .unwrap_or_default();

        let collection_return_str = requires_collection
            .then_some(
                "
        sui::test_scenario::return_shared(collection);",
            )
            .unwrap_or_default();

        let mut tests = format!(
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
        );{collection_take_str}

        let warehouse = ob_launchpad::warehouse::new<{type_name}>(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_warehouse(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,{collection_param_str}
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);{collection_return_str}
        sui::test_scenario::end(scenario);
    }}");

        tests.push_str(&self.nft.write_dynamic_tests(collection));
        tests.push_str(&self.nft.write_burn_tests(collection));

        tests
    }

    /// Higher level method responsible for generating Move code from the
    /// struct `Schema` and dump it into a default folder
    /// `../sources/examples/<module_name>.move` or custom folder defined by
    /// the caller.
    pub fn write_move<W: std::io::Write>(
        &self,
        mut output: W,
    ) -> Result<(), GutenError> {
        let witness = self.collection().witness_name();
        let package_name = self.package_name();

        let type_declarations = self.settings.write_type_declarations();
        let init_function = self.write_init_fn();
        let entry_functions = self.write_entry_fns();
        let nft_struct = self.nft.write_struct();

        let tests = self.write_tests();

        let res = format!(
            include_str!("../templates/template.move"),
            witness = witness,
            package_name = package_name,
            type_declarations = type_declarations,
            init_function = init_function,
            entry_functions = entry_functions,
            nft_struct = nft_struct,
            tests = tests
        );

        output.write_all(res.as_bytes())?;

        Ok(())
    }

    pub fn write_move_toml<W: std::io::Write>(
        &self,
        mut output: W,
    ) -> Result<(), GutenError> {
        let package_name = self.package_name();

        let res = format!(
            include_str!("../templates/Move.toml"),
            package_name = package_name,
        );

        output.write_all(res.as_bytes())?;

        Ok(())
    }
}
