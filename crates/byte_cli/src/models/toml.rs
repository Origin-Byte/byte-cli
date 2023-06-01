use anyhow::anyhow;
use convert_case::{Case, Casing};
use gutenberg::models::Address;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap},
    fmt,
    marker::PhantomData,
};

use crate::{consts::OB_PACKAGES, err::CliError};

use super::dependencies::{Contract, ContractRef, PackageMap};

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

#[derive(Deserialize, Debug)]
pub struct MoveToml {
    pub package: Package,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: Version,
    pub published_at: Option<Address>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
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
    ) -> Vec<ContractRef> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).unwrap_or_else(|| {
                    panic!("Could not find Package Name {} in PackageMap", name)
                });

                get_contract_ref(specs, dep_pack)
            })
            .collect::<Vec<ContractRef>>();

        dep_ids
    }

    pub fn get_contracts<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<&'a Contract> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map.0.get(name).unwrap_or_else(|| {
                    panic!("Could not find Package Name {} in PackageMap", name)
                });

                get_contract(specs, dep_pack)
            })
            .collect::<Vec<&'a Contract>>();

        dep_ids
    }

    pub fn get_contracts_with_fall_back<'a>(
        &'a self,
        package_map: &'a PackageMap,
        fall_back: &'a BuildInfo,
    ) -> Vec<&'a Contract> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = package_map
                    .0
                    .get(name)
                    .ok_or_else(|| {
                        fall_back
                            .packages
                            .ob_packages
                            .get(name)
                            .expect("Could not find package ID")
                    })
                    .unwrap();

                get_contract(specs, dep_pack)
            })
            .collect::<Vec<&'a Contract>>();

        dep_ids
    }
}

pub fn get_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> &'a Contract {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .unwrap_or_else(|| panic!("Could not find rev {} in version map", rev))
        .1
}

pub fn get_version_and_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> (&'a Version, &'a Contract) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .unwrap_or_else(|| panic!("Could not find rev {} in version map", rev))
}

pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_contract_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

pub fn get_contract_ref(
    dependency: &Dependency,
    versions: &BTreeMap<Version, Contract>,
) -> ContractRef {
    let (_, contract) =
        get_version_and_contract_from_rev(versions, &dependency.rev);

    ContractRef {
        path: dependency.clone(),
        object_id: contract.contract_ref.object_id.clone(),
    }
}

pub fn get_contract<'a>(
    dependency: &'a Dependency,
    versions: &'a BTreeMap<Version, Contract>,
) -> &'a Contract {
    let (_, contract) =
        get_version_and_contract_from_rev(versions, &dependency.rev);

    contract
}

pub fn get_dependencies_to_update<'a>(
    deps: &'a [&'a Contract],
    package_map: &'a PackageMap,
) -> Vec<&'a Contract> {
    let mut to_update: Vec<&'a Contract> = vec![];

    deps.iter().for_each(|contract| {
        if let Some(update) = get_updated_dependency(contract, package_map) {
            to_update.push(update);
        }
    });

    to_update
}

pub fn get_updated_dependency<'a>(
    dep: &'a Contract,
    package_map: &'a PackageMap,
) -> Option<&'a Contract> {
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
) -> Result<Version, CliError> {
    for (_, version_map) in package_map.0.iter() {
        let search_result = version_map.iter().find(|(_, contract)| {
            contract.contract_ref.object_id == *object_id
        });

        if let Some(search_result) = search_result {
            return Ok(*search_result.0);
        }
    }

    Err(CliError::from(anyhow!(
        "Unable to find object ID in package map"
    )))
}

struct VersionVisitor {
    marker: PhantomData<fn() -> Version>,
}

impl VersionVisitor {
    fn new() -> Self {
        VersionVisitor {
            marker: PhantomData,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn from_string(string: &str) -> Result<Self, CliError> {
        let version: Vec<&str> = string.split('.').collect();

        if version.len() != 3 {
            return Err(CliError::from(anyhow!(
                "Version semantics is incorrect"
            )));
        }

        let version = version
            .iter()
            .map(|v| {
                let v_string = v.to_string();
                v_string.parse::<u8>().unwrap()
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

impl<'de> Visitor<'de> for VersionVisitor {
    type Value = Version;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string containing the package version in semantic versioning format xx.yy.zz")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let version: Vec<&str> = s.split('.').collect();

        if version.len() != 3 {
            return Err(de::Error::invalid_value(Unexpected::Str(s), &self));
        }

        let version = version
            .iter()
            .map(|v| {
                let v_string = v.to_string();
                v_string.parse::<u8>().unwrap()
            })
            .collect::<Vec<u8>>();

        Ok(Version {
            major: version[0],
            minor: version[1],
            patch: version[2],
        })
    }
}

// This is the trait that informs Serde how to deserialize Version.
impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate VersionVisitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of Version.
        deserializer.deserialize_str(VersionVisitor::new())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_ord = self.major.cmp(&other.major);
        if major_ord != Ordering::Equal {
            return major_ord;
        }

        let minor_ord = self.minor.cmp(&other.minor);
        if minor_ord != Ordering::Equal {
            return minor_ord;
        }

        self.patch.cmp(&other.patch)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let major_ord = self.major.partial_cmp(&other.major);
        if major_ord.unwrap() != Ordering::Equal {
            return major_ord;
        }

        let minor_ord = self.minor.partial_cmp(&other.minor);
        if minor_ord.unwrap() != Ordering::Equal {
            return minor_ord;
        }

        self.patch.partial_cmp(&other.patch)
    }
}

#[cfg(test)]
mod test_version {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_order() -> Result<()> {
        let version_a = Version::from_string(&String::from("1.0.1"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("1.1.0"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("1.0.0"))?;
        let version_b = Version::from_string(&String::from("1.0.0"))?;
        assert!(version_a == version_b);

        let version_a = Version::from_string(&String::from("2.0.0"))?;
        let version_b = Version::from_string(&String::from("1.5.0"))?;
        assert!(version_a > version_b);

        let version_a = Version::from_string(&String::from("0.0.30"))?;
        let version_b = Version::from_string(&String::from("0.5.0"))?;
        assert!(version_a < version_b);

        Ok(())
    }
}
