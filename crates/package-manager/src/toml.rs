use anyhow::{anyhow, Result};
use console::style;
use gutenberg::models::Address;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::{
    move_lib::{Dependency, LibSpecs, MoveLib, Package, PackageMap},
    version::Version,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct MoveToml {
    pub package: Package,
    pub dependencies: HashMap<String, Dependency>,
    pub addresses: HashMap<String, Address>,
}

impl MoveToml {
    pub fn sanitize_output(&mut self) {
        self.dependencies
            .iter_mut()
            .for_each(|(_, dep)| dep.sanitize_subdir());
    }

    pub fn get_dependency_ids<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<&'a Address> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageMap",
                        name
                    )
                    .as_str(),
                );

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect::<Vec<&'a Address>>();

        dep_ids
    }

    pub fn get_contract_refs<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<LibSpecs> {
        self.dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageMap",
                        name
                    )
                    .as_str(),
                );

                get_contract_ref(specs, dep_pack)
            })
            .collect::<Vec<LibSpecs>>()
    }

    pub fn get_contracts<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<&'a MoveLib> {
        self.dependencies
            .iter()
            .filter_map(|(name, specs)| {
                let dep_pack = package_map.0.get(name);

                if let Some(pack) = dep_pack {
                    Some(get_contract(specs, pack))
                } else {
                    println!("{} Skipping Package {:?}, could not find it in the Package Registry", style("Warning ").yellow().bold(), name);
                    None
                }
            })
            .collect::<Vec<&'a MoveLib>>()
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
            .for_each(|(dep_name, mut dep, dep_version)| {
                println!(
                    "{}{}",
                    style("Updated ").green().bold().on_bright(),
                    format!("{} to version {}", dep_name, dep_version).as_str()
                );

                dep.sanitize_subdir();

                self.dependencies.insert(dep_name, dep);
            });
    }

    pub fn get_toml(
        name: &str,
        package_map: &PackageMap,
        dep_names: &Vec<String>,
        ext_dep_names: &Vec<String>,
        version: &Version,
    ) -> Result<Self> {
        let empty_addr = Address::new(String::from("0x0"))?;

        let mut dependencies =
            get_dependencies(package_map, dep_names, &version);

        // Inserts Sui and Originmate
        ext_dep_names.iter().for_each(|dep_name| {
            dependencies.insert(
                dep_name.clone(),
                get_ext_dep_from_protocol(
                    dep_name.clone(),
                    package_map,
                    version,
                ),
            );
        });

        let toml = MoveToml {
            package: Package {
                name: name.to_string(),
                version: Version::from_string("1.0.0")?,
                published_at: Some(empty_addr.clone()),
            },
            dependencies,
            addresses: HashMap::from([(String::from(name), empty_addr)]),
        };

        Ok(toml)
    }

    pub fn get_toml_latest(
        name: &str,
        package_map: &PackageMap,
        dep_names: &Vec<String>,
        ext_dep_names: &Vec<String>,
    ) -> Result<Self> {
        // Oath of honor --> Monolitic release (for now)
        let version = get_latest_protocol_version(
            &String::from("NftProtocol"),
            package_map,
        );

        MoveToml::get_toml(name, package_map, dep_names, ext_dep_names, version)
    }

    pub fn get_dependency<'a>(
        self: &'a Self,
        dep_name: &'a str,
    ) -> &'a Dependency {
        // Fetch available versions by package name
        let dependency = self.dependencies.get(dep_name).expect(
            format!("Could not find Dependency Name {} in Move.toml", dep_name)
                .as_str(),
        );

        dependency
    }
}

pub fn get_dependencies(
    package_map: &PackageMap,
    dep_names: &Vec<String>,
    version: &Version,
) -> HashMap<String, Dependency> {
    let deps = dep_names
        .iter()
        .map(|dep_name| {
            (
                dep_name.clone(),
                get_dependency(dep_name, package_map, version)
                    .contract_ref
                    .path
                    .clone(),
            )
        })
        .collect::<HashMap<String, Dependency>>();

    deps
}

// i.e. Sui or Originmate
fn get_ext_dep_from_protocol(
    ext_dep: String,
    package_map: &PackageMap,
    version: &Version,
) -> Dependency {
    let protocol_versions =
        package_map.0.get(&String::from("NftProtocol")).expect(
            format!("Could not find Package Name {} in PackageMap", &ext_dep)
                .as_str(),
        );

    protocol_versions
        .get(version)
        .unwrap()
        .dependencies
        .get(&ext_dep)
        .expect(format!("Unable to fetch {} dependency", ext_dep).as_str())
        .path
        .clone()
}

pub fn get_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, MoveLib>,
    rev: &'a String,
) -> &'a MoveLib {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
        .1
}

pub fn get_version_and_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, MoveLib>,
    rev: &'a String,
) -> (&'a Version, &'a MoveLib) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
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
    let versions = package_map.0.get(&dep.package.name).expect(
        format!(
            "Could not find Package Name {} in PackageMap",
            &dep.package.name
        )
        .as_str(),
    );

    let latest_version = versions
        .keys()
        .max()
        // This error should not occur
        .expect(
            format!(
                "Unexpected error: Unable to retrieve latest version of {}",
                &dep.package.name
            )
            .as_str(),
        );

    let latest = versions.get(latest_version).unwrap();

    (dep.package.version != latest.package.version).then_some(latest)
}

fn get_latest_protocol_version<'a>(
    dep_name: &String,
    package_map: &'a PackageMap,
) -> &'a Version {
    // Fetch available versions by package name
    let versions = package_map.0.get(dep_name).expect(
        format!("Could not find Package Name {} in PackageMap", dep_name)
            .as_str(),
    );

    versions
        .keys()
        .max()
        // This error should not occur
        .expect(
            format!(
                "Unexpected error: Unable to retrieve latest version of {}",
                dep_name
            )
            .as_str(),
        )
}

pub fn get_dependency<'a>(
    dep_name: &String,
    package_map: &'a PackageMap,
    version: &Version,
) -> &'a MoveLib {
    // Fetch available versions by package name
    let versions = package_map.0.get(dep_name).expect(
        format!("Could not find Package Name {} in PackageMap", dep_name)
            .as_str(),
    );

    let dependency = versions
        .get(version)
        .expect(format!("Unable to fetch {} v{}", dep_name, version).as_str());

    dependency
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

/// This function is here because Toml serialiser seems to be
/// failing to add a vertical space between the tables `package` and `dependencies`
pub fn add_vertical_spacing(input: &str) -> String {
    let re = Regex::new(r"(?m)^(published-at.*)").unwrap();
    re.replace_all(input, "$1\n").to_string()
}
