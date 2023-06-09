use anyhow::{anyhow, Result};
use console::style;
use gutenberg::models::Address;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::{
    move_lib::{Dependency, LibSpecs, MoveLib, Package, PackageMap},
    version::Version,
};

#[derive(Deserialize, Debug)]
pub struct MoveToml {
    pub package: Package,
    pub dependencies: HashMap<String, Dependency>,
}

impl MoveToml {
    pub fn get_dependency_ids<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<&'a Address> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).unwrap_or_else(|| {
                    panic!("Could not find Package Name {} in PackageMap", name)
                });

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect::<Vec<&'a Address>>();

        dep_ids
    }

    pub fn get_contract_refs<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<LibSpecs> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).unwrap_or_else(|| {
                    panic!("Could not find Package Name {} in PackageMap", name)
                });

                get_contract_ref(specs, dep_pack)
            })
            .collect::<Vec<LibSpecs>>();

        dep_ids
    }

    pub fn get_contracts<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<&'a MoveLib> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).unwrap_or_else(|| {
                    panic!("Could not find Package Name {} in PackageMap", name)
                });

                get_contract(specs, dep_pack)
            })
            .collect::<Vec<&'a MoveLib>>();

        dep_ids
    }

    pub fn update_toml(&mut self, package_map: &PackageMap) {
        let dependencies = self.get_contracts(package_map);

        let to_update = get_dependencies_to_update(&dependencies, package_map);

        let mut updated_deps = to_update
            .iter()
            .map(|dep| {
                (
                    dep.package.name.clone(),
                    dep.contract_ref.path.clone(),
                    dep.package.version,
                )
            })
            .collect::<Vec<(String, Dependency, Version)>>();

        updated_deps
            .drain(..)
            .for_each(|(dep_name, dep, dep_version)| {
                println!(
                    "{}{}",
                    style("Updated ").green().bold().on_bright(),
                    format!("{} to version {}", dep_name, dep_version).as_str()
                );

                self.dependencies.insert(dep_name, dep);
            });
    }
}

pub fn get_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, MoveLib>,
    rev: &'a String,
) -> &'a MoveLib {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .unwrap_or_else(|| panic!("Could not find rev {} in version map", rev))
        .1
}

pub fn get_version_and_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, MoveLib>,
    rev: &'a String,
) -> (&'a Version, &'a MoveLib) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .unwrap_or_else(|| panic!("Could not find rev {} in version map", rev))
}

pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, MoveLib>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_contract_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

pub fn get_contract_ref(
    dependency: &Dependency,
    versions: &BTreeMap<Version, MoveLib>,
) -> LibSpecs {
    let (_, contract) =
        get_version_and_contract_from_rev(versions, &dependency.rev);

    LibSpecs {
        path: dependency.clone(),
        object_id: contract.contract_ref.object_id.clone(),
    }
}

pub fn get_contract<'a>(
    dependency: &'a Dependency,
    versions: &'a BTreeMap<Version, MoveLib>,
) -> &'a MoveLib {
    let (_, contract) =
        get_version_and_contract_from_rev(versions, &dependency.rev);

    contract
}

pub fn get_dependencies_to_update<'a>(
    deps: &'a [&'a MoveLib],
    package_map: &'a PackageMap,
) -> Vec<&'a MoveLib> {
    let mut to_update: Vec<&'a MoveLib> = vec![];

    deps.iter().for_each(|contract| {
        if let Some(update) = get_updated_dependency(contract, package_map) {
            to_update.push(update);
        }
    });

    to_update
}

pub fn get_updated_dependency<'a>(
    dep: &'a MoveLib,
    package_map: &'a PackageMap,
) -> Option<&'a MoveLib> {
    // Fetch available versions by package name
    let versions = package_map.0.get(&dep.package.name).unwrap_or_else(|| {
        panic!(
            "Could not find Package Name {} in PackageMap",
            &dep.package.name
        )
    });

    let latest_version = versions
        .keys()
        .max()
        // This error should not occur
        .expect("Failed while retrieving latest version");

    let latest = versions.get(latest_version).unwrap();

    if dep.package.version == latest.package.version {
        None
    } else {
        Some(latest)
    }
}

pub fn get_version_from_object_id(
    object_id: &Address,
    package_map: &PackageMap,
) -> Result<Version> {
    for (_, version_map) in package_map.0.iter() {
        let search_result = version_map.iter().find(|(_, contract)| {
            contract.contract_ref.object_id == *object_id
        });

        if let Some(search_result) = search_result {
            return Ok(*search_result.0);
        }
    }

    Err(anyhow!("Unable to find object ID in package map"))
}
