use anyhow::Result;
use console::style;
use gutenberg::models::Address;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::{
    pkg::{GitPath, Package, PkgInfo, PkgPath, PkgRegistry},
    version::Version,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct MoveToml {
    pub package: Package,
    pub dependencies: HashMap<String, GitPath>,
    pub addresses: HashMap<String, Address>,
}

impl MoveToml {
    pub fn sanitize_output(&mut self) {
        self.dependencies
            .iter_mut()
            .for_each(|(_, dep)| dep.sanitize_subdir());
    }

    pub fn pkg_info_list<'a>(
        &'a self,
        pkg_registry: &'a PkgRegistry,
    ) -> &'a BTreeMap<Version, PkgInfo> {
        pkg_registry.0.get(&self.package.name_pascal()).expect(
            format!(
                "Could not find package '{}' in Package Registry",
                self.package.name_pascal()
            )
            .as_str(),
        )
    }

    pub fn pkg_info<'a>(
        &'a self,
        pkg_registry: &'a PkgRegistry,
    ) -> &'a PkgInfo {
        let info_list = self.pkg_info_list(pkg_registry);

        info_list.get(&self.package.version).expect(
            format!(
                "Could not find version '{}' for Package '{}'",
                self.package.version,
                self.package.name_pascal()
            )
            .as_str(),
        )
    }

    pub fn dependency_pkg_infos<'a>(
        &'a self,
        pkg_registry: &'a PkgRegistry,
    ) -> Vec<&'a PkgInfo> {
        self.dependencies
            .iter()
            .filter_map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name);

                if let Some(pack) = dep_pack {
                    Some(get_pkg_info(specs, pack))
                } else {
                    println!("{} Skipping Package {:?}, could not find it in the Package Registry", style("Warning ").yellow().bold(), name);
                    None
                }
            })
            .collect::<Vec<&'a PkgInfo>>()
    }

    pub fn get_dependency_ids<'a>(
        &'a self,
        pkg_registry: &'a PkgRegistry,
    ) -> Vec<&'a Address> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PkgRegistry",
                        name
                    )
                    .as_str(),
                );

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect::<Vec<&'a Address>>();

        dep_ids
    }

    pub fn update_toml(&mut self, pkg_registry: &PkgRegistry) {
        let dependencies = self.dependency_pkg_infos(pkg_registry);

        let to_update = pkg_registry.get_pkgs_to_update(&dependencies);

        let mut updated_deps = to_update
            .iter()
            .map(|dep| {
                (
                    dep.package.name.clone(),
                    dep.contract_ref.path.clone(),
                    dep.package.version,
                )
            })
            .collect::<Vec<(String, GitPath, Version)>>();

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
        pkg_registry: &PkgRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
        version: &Version,
    ) -> Result<Self> {
        let empty_addr = Address::new(String::from("0x0"))?;

        let mut dependencies = pkg_registry.get_pkgs_git(dep_names, version);

        // Inserts Sui and Originmate
        ext_dep_names.iter().for_each(|dep_name| {
            dependencies.insert(
                dep_name.clone(),
                pkg_registry
                    .get_ext_dep_from_protocol(dep_name.as_str(), version),
            );
        });

        let toml = MoveToml {
            package: Package {
                name: name.to_string(),
                version: Version::from_str("1.0.0")?,
                published_at: Some(empty_addr.clone()),
            },
            dependencies,
            addresses: HashMap::from([(String::from(name), empty_addr)]),
        };

        Ok(toml)
    }

    pub fn get_toml_latest(
        name: &str,
        pkg_registry: &PkgRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
    ) -> Result<Self> {
        // Oath of honor --> Monolitic release (for now)
        let version =
            pkg_registry.get_latest_version(&String::from("NftProtocol"));

        MoveToml::get_toml(
            name,
            pkg_registry,
            dep_names,
            ext_dep_names,
            version,
        )
    }

    pub fn get_dependency<'a>(&'a self, dep_name: &'a str) -> &'a GitPath {
        // Fetch available versions by package name
        let dependency = self.dependencies.get(dep_name).expect(
            format!("Could not find GitPath Name {} in Move.toml", dep_name)
                .as_str(),
        );

        dependency
    }
}

pub fn get_pkg_info<'a>(
    dependency: &'a GitPath,
    versions: &'a BTreeMap<Version, PkgInfo>,
) -> &'a PkgInfo {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    contract
}

pub fn get_version_and_pkg_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PkgInfo>,
    rev: &'a String,
) -> (&'a Version, &'a PkgInfo) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
}

pub fn get_pkg_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PkgInfo>,
    rev: &'a String,
) -> &'a PkgInfo {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
        .1
}

pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, PkgInfo>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_pkg_info_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

pub fn get_contract_ref(
    dependency: &GitPath,
    versions: &BTreeMap<Version, PkgInfo>,
) -> PkgPath {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    PkgPath {
        path: dependency.clone(),
        object_id: contract.contract_ref.object_id.clone(),
    }
}

/// This function is here because Toml serialiser seems to be
/// failing to add a vertical space between the tables `package` and `dependencies`
pub fn add_vertical_spacing(input: &str) -> String {
    let re = Regex::new(r"(?m)^(published-at.*)")
        .expect("Failed to read `published-at` field");
    re.replace_all(input, "$1\n").to_string()
}
