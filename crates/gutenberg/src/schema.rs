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

    pub fn write_init_fn(&self) -> Result<String, GutenError> {
        let domains = self.collection.write_domains();

        let feature_domains =
            self.settings.write_feature_domains(&self.collection)?;

        let transfer_fns = self
            .settings
            .write_transfer_fns(self.collection.creators.first().unwrap())?;

        Ok(format!(
            "    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let (mint_cap, collection) = nft_protocol::collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness<{witness}, Witness>(&Witness {{}});
{domains}{feature_domains}{transfer_fns}    }}",
            witness = self.witness_name()
        ))
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

        let init_fn = self.write_init_fn()?;

        let entry_fns = self.write_entry_fns();

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
