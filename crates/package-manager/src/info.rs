use convert_case::{Case, Casing};
use gutenberg_types::models::address::Address;
use serde::Deserialize;
use std::collections::BTreeMap;

use crate::OB_PACKAGES;

/// Struct to hold build information.
///
/// This struct is designed to deserialize and contain information about a build,
/// specifically the compiled package information.
///
/// # Fields
/// * `packages` - Contains the compiled package information, aliased as "compiled_package_info"
///                during deserialization.
#[derive(Deserialize, Debug)]
pub struct BuildInfo {
    #[serde(alias = "compiled_package_info")]
    pub packages: CompiledPackageInfo,
}

/// Represents the compiled package information.
///
/// This struct is used to deserialize and manage information about compiled packages,
/// specifically focusing on their names and associated addresses.
///
/// # Fields
/// * `package_name` - The name of the package.
/// * `ob_packages` - A `BTreeMap` mapping package names to their blockchain addresses.
///                  These names are aliased as "address_alias_instantiation" during deserialization.
#[derive(Deserialize, Debug)]
pub struct CompiledPackageInfo {
    pub package_name: String,
    #[serde(alias = "address_alias_instantiation")]
    pub ob_packages: BTreeMap<String, Address>,
}

impl CompiledPackageInfo {
    // Note: Ideally this function should run at deserialization time
    /// Removes `ob_` prefix from package names within the `ob_packages` map.
    ///
    /// This function is intended to clean the package names by removing the `ob_` prefix.
    /// It should ideally be run during the deserialization process. The function retains
    /// only those package names that are contained within the `OB_PACKAGES` after removal
    /// of the prefix and conversion to Pascal case.
    pub fn remove_ob_prefix(&mut self) {
        self.ob_packages.retain(|name, _| {
            // Removes `ob_` prefix from the package names
            // Note: Susceptible to false positives if a
            // word ends in `ob` and another word comes after
            let name = name.replace("ob_", "").as_str().to_case(Case::Pascal);
            OB_PACKAGES.contains(&name.as_str())
        })
    }

    // Note: Ideally this function should run at deserialization time
    /// Converts package names within the `ob_packages` map to their canonical form.
    ///
    /// This function transforms the package names by removing the `ob_` prefix and converting
    /// them to Pascal case. It's designed to standardize the package names to a canonical form
    /// and should ideally be run during deserialization.
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
