//! Module containing the core logic to parse the `config.yaml` file into a
//! struct `Schema`, acting as an intermediate data structure, to write
//! the associated Move module and dump into a default or custom folder defined
//! by the caller.
use std::path::PathBuf;

use crate::models::{collection::CollectionData, nft::NftData};
use crate::{normalize_type, ContractFile};
use serde::{Deserialize, Serialize};

/// Struct that acts as an intermediate data structure representing the yaml
/// configuration of the NFT collection.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    /// The named address that the module is published under
    package_name: String,
    #[serde(default)]
    collection: CollectionData,
    nft: NftData,
}

impl Schema {
    pub fn new(
        package_name: String,
        collection: CollectionData,
        nft: NftData,
    ) -> Schema {
        Schema {
            package_name,
            collection,
            nft,
        }
    }

    pub fn package_name(&self) -> String {
        // Since `Schema.package_name` can be deserialized from an untrusted
        // source it's fields must be escaped when preparing for display.
        normalize_type(&self.package_name).to_lowercase()
    }

    pub fn collection(&self) -> &CollectionData {
        &self.collection
    }

    pub fn nft(&self) -> &NftData {
        &self.nft
    }

    /// Disables features that should not be enabled in demo mode
    pub fn enforce_demo(&mut self) {
        self.nft.enforce_demo();
        self.collection.enforce_demo();
    }

    pub fn write_move_defs(&self) -> String {
        self.nft.write_move_defs(&self.collection)
    }

    pub fn write_tests(&self) -> String {
        let type_name = self.nft().type_name();
        let witness_name = self.nft().witness_name();
        let collection_data = self.collection();

        let mut tests_str = format!(
            "

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {{
        let scenario = sui::test_scenario::begin(CREATOR);

        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(sui::test_scenario::has_most_recent_shared<nft_protocol::collection::Collection<{type_name}>>(), 0);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario, CREATOR,
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        sui::test_scenario::end(scenario);
    }}");

        tests_str.push_str(&self.nft.write_move_tests(collection_data));

        tests_str
    }

    /// Higher level method responsible for generating Move code from the
    /// struct `Schema` and dump it into a default folder
    /// `../sources/examples/<module_name>.move` or custom folder defined by
    /// the caller.
    pub fn write_move(&self) -> ContractFile {
        let witness_name = self.nft().witness_name();
        let module_name = self.nft().module_name();
        let package_name = self.package_name();

        let definitions = self.write_move_defs();
        let tests = self.write_tests();

        let content = format!(
            "module {package_name}::{module_name} {{
    /// One time witness is only instantiated in the init method
    struct {witness_name} has drop {{}}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {{}}{definitions}{tests}
}}
"
        );

        ContractFile {
            path: PathBuf::from("sources").join(format!("{module_name}.move")),
            content,
        }
    }

    pub fn write_move_toml(&self) -> ContractFile {
        let content = format!(
            include_str!("../templates/Move.toml"),
            package_name = self.package_name(),
        );

        ContractFile {
            path: PathBuf::from("Move.toml"),
            content,
        }
    }
}
