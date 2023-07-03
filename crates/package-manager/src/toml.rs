use super::Address;
use anyhow::anyhow;
use console::style;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
};

use crate::{
    package::{GitPath, Package, PackageInfo, PackagePath, PackageRegistry},
    version::Version,
};

#[derive(Deserialize, Debug, Serialize)]
pub struct MoveToml {
    package: Package,
    dependencies: HashMap<String, GitPath>,
    addresses: HashMap<String, Address>,
}

impl MoveToml {
    pub fn new(
        package: Package,
        dependencies: HashMap<String, GitPath>,
        addresses: HashMap<String, Address>,
    ) -> Self {
        Self {
            package,
            dependencies,
            addresses,
        }
    }

    pub fn sanitize_output(&mut self) {
        self.dependencies
            .iter_mut()
            .for_each(|(_, dep)| dep.sanitize_subdir());
    }

    pub fn pkg_info_list<'a>(
        &self,
        pkg_registry: &'a PackageRegistry,
    ) -> Result<&'a BTreeMap<Version, PackageInfo>, anyhow::Error> {
        let name = self.package.name();
        pkg_registry.0.get(&name).ok_or_else(|| {
            anyhow!("Could not find package '{name}' in Package Registry")
        })
    }

    pub fn pkg_info<'a>(
        &self,
        pkg_registry: &'a PackageRegistry,
    ) -> Result<&'a PackageInfo, anyhow::Error> {
        let version = self.package.version();
        let info_list = self.pkg_info_list(pkg_registry)?;

        info_list.get(&version).ok_or_else(|| {
            anyhow!(
                "Could not find version '{version}' for Package '{}'",
                self.package.name()
            )
        })
    }

    pub fn dependency_pkg_infos<'a>(
        &'a self,
        pkg_registry: &'a PackageRegistry,
    ) -> Vec<&'a PackageInfo> {
        self.dependencies
            .iter()
            .filter_map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name);

                if let Some(pack) = dep_pack {
                    Some(get_package_info(specs, pack))
                } else {
                    println!("{} Skipping Package {:?}, could not find it in the Package Registry", style("Warning ").yellow().bold(), name);
                    None
                }
            })
            .collect()
    }

    pub fn get_dependency_ids<'a>(
        &'a self,
        pkg_registry: &'a PackageRegistry,
    ) -> Vec<&'a Address> {
        self.dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageRegistry",
                        name
                    )
                    .as_str(),
                );

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect()
    }

    pub fn update_toml(&mut self, pkg_registry: &PackageRegistry) {
        let dependencies = self.dependency_pkg_infos(pkg_registry);

        let to_update = pkg_registry.get_packages_to_update(&dependencies);

        let mut updated_deps = to_update
            .iter()
            .map(|dep| {
                (
                    dep.package.name(),
                    dep.contract_ref.path.clone(),
                    dep.package.version().clone(),
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
        pkg_registry: &PackageRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
        version: &Version,
    ) -> Result<Self, anyhow::Error> {
        let empty_addr = Address::new("0x0")?;

        let mut dependencies =
            pkg_registry.get_packages_git(dep_names, version)?;

        // Inserts Sui and Originmate
        ext_dep_names
            .iter()
            .try_for_each::<_, Result<(), anyhow::Error>>(|dep_name| {
                let ext_dep = pkg_registry
                    .get_ext_dep_from_protocol(dep_name.as_str(), version)?;

                dependencies.insert(dep_name.clone(), ext_dep);

                Ok(())
            })?;

        let toml = MoveToml {
            package: Package::new(
                name.to_string(),
                Version::from_str("1.0.0")?,
                Some(empty_addr.clone()),
            ),
            dependencies,
            addresses: HashMap::from([(String::from(name), empty_addr)]),
        };

        Ok(toml)
    }

    pub fn get_toml_latest(
        name: &str,
        pkg_registry: &PackageRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
    ) -> Result<Self, anyhow::Error> {
        // Oath of honor --> Monolitic release (for now)
        let version =
            pkg_registry.get_latest_version(&String::from("NftProtocol"))?;

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

    pub fn to_value(&self) -> Result<toml::Value, toml::ser::Error> {
        toml::Value::try_from(self)
    }

    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        let mut toml_string = toml::to_string_pretty(self)?;
        toml_string = add_vertical_spacing(toml_string.as_str());

        Ok(toml_string)
    }
}

pub fn get_package_info<'a>(
    dependency: &'a GitPath,
    versions: &'a BTreeMap<Version, PackageInfo>,
) -> &'a PackageInfo {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    contract
}

pub fn get_version_and_pkg_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> (&'a Version, &'a PackageInfo) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
}

pub fn get_package_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> &'a PackageInfo {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
        .1
}

pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_package_info_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

pub fn get_contract_ref(
    dependency: &GitPath,
    versions: &BTreeMap<Version, PackageInfo>,
) -> PackagePath {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    PackagePath {
        path: dependency.clone(),
        object_id: contract.contract_ref.object_id.clone(),
    }
}

/// This function is here because Toml serializer seems to be
/// failing to add a vertical space between the tables `package` and `dependencies`
pub fn add_vertical_spacing(input: &str) -> String {
    let re = Regex::new(r"(?m)^(published-at.*)")
        .expect("Failed to read `published-at` field");
    re.replace_all(input, "$1\n").to_string()
}
