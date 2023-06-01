use anyhow::anyhow;
use convert_case::{Case, Casing};
use gutenberg::models::Address;
use serde::{
    de::{self, MapAccess, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    collections::{BTreeMap, HashMap},
    ffi::OsStr,
    fmt,
    marker::PhantomData,
    process::{Command, Stdio},
};

use crate::{cli::Cli, consts::OB_PACKAGES, err::CliError, io::LocalWrite};

use super::dependencies::{Contract, PackageMap};

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
    pub fn filter_for_originbyte(&mut self) {
        self.ob_packages.retain(|name, _| {
            // Removes `ob_` prefix from the package names
            // Note: Susceptible to false positives if a
            // word ends in `ob` and another word comes after
            let name = name.replace("ob_", "").as_str().to_case(Case::Pascal);
            OB_PACKAGES.contains(&name.as_str())
        })
    }
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

#[derive(Deserialize, Debug)]
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
                let dep_pack = package_map.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageMap",
                        name,
                    )
                    .as_str(),
                );

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect::<Vec<&'a Address>>();

        dep_ids
    }

    pub fn get_dependency_ids_and_versions<'a>(
        &'a self,
        package_map: &'a PackageMap,
    ) -> Vec<(&'a Address, Version)> {
        let dep_ids = self
            .dependencies
            .iter()
            .map(|(name, specs)| {
                println!("Crackers! Depend");
                let dep_pack = package_map.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageMap",
                        name,
                    )
                    .as_str(),
                );

                get_object_id_and_version_from_rev(dep_pack, &specs.rev)
            })
            .collect::<Vec<(&'a Address, Version)>>();

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
        .expect(format!("Could not find rev {} in version map", rev).as_str())
        .1
}

pub fn get_version_and_contract_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> (&'a Version, &'a Contract) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
}

pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_contract_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

pub fn get_object_id_and_version_from_rev<'a>(
    versions: &'a BTreeMap<Version, Contract>,
    rev: &'a String,
) -> (&'a Address, Version) {
    let (version, contract) = get_version_and_contract_from_rev(versions, rev);

    (&contract.contract_ref.object_id, *version)
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

// TODO: Ord and PartialOrd may need specific implementation
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub struct Version {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl Version {
    pub fn from_string(string: &String) -> Result<Self, CliError> {
        let version: Vec<&str> = string.split(".").collect();

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
        let version: Vec<&str> = s.split(".").collect();

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
