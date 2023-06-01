use std::collections::BTreeMap;

use convert_case::{Case, Casing};
use gutenberg::models::Address;
use serde::Deserialize;

use crate::OB_PACKAGES;

#[derive(Deserialize, Debug)]
pub struct BuildInfo {
    #[serde(alias = "compiled_package_info")]
    pub packages: CompiledPackageInfo,
}

#[derive(Deserialize, Debug)]
pub struct CompiledPackageInfo {
    pub package_name: String,
    #[serde(alias = "address_alias_instantiation")]
    pub ob_packages: BTreeMap<String, Address>,
}

impl CompiledPackageInfo {
    // Ideally this function should run at deserialization time
    pub fn filter_for_originbyte(&mut self) {
        self.ob_packages.retain(|name, _| {
            // Removes `ob_` prefix from the package names
            // Note: Susceptible to false positives if a
            // word ends in `ob` and another word comes after
            let name = name.replace("ob_", "").as_str().to_case(Case::Pascal);
            OB_PACKAGES.contains(&name.as_str())
        })
    }

    // Ideally this function should run at deserialization time
    pub fn make_name_canonical(&mut self) {
        let canonical = self
            .ob_packages
            .iter()
            .map(|(name, address)| {
                let name =
                    name.replace("ob_", "").as_str().to_case(Case::Pascal);
                (name, address.clone())
            })
            .collect::<BTreeMap<String, Address>>();

        self.ob_packages = canonical;
    }
}
