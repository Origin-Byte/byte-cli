use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use gutenberg::models::Address;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::version::Version;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgRegistry(pub BTreeMap<String, BTreeMap<Version, PkgInfo>>);

impl PkgRegistry {
    pub fn get_object_id_from_ref(
        &self,
        pkg_name: String,
        rev: String,
    ) -> &Address {
        let versions = &self.0.get(&pkg_name).unwrap();

        let (_, metadata) = versions
            .iter()
            .find(|(_v, metadata)| metadata.contract_ref.path.rev == rev)
            .unwrap();

        metadata.package.published_at.as_ref().unwrap()
    }

    pub fn get_dependency<'a>(
        &'a self,
        dep_name: &'a String,
        version: &'a Version,
    ) -> &'a PkgInfo {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).expect(
            format!("Could not find Package Name {} in PkgRegistry", dep_name)
                .as_str(),
        );

        let dependency = versions.get(version).expect(
            format!("Unable to fetch {} v{}", dep_name, version).as_str(),
        );

        dependency
    }

    pub fn get_dependencies(
        &self,
        dep_names: &Vec<String>,
        version: &Version,
    ) -> HashMap<String, GitPath> {
        let deps = dep_names
            .iter()
            .map(|dep_name| {
                (
                    dep_name.clone(),
                    self.get_dependency(dep_name, version)
                        .contract_ref
                        .path
                        .clone(),
                )
            })
            .collect::<HashMap<String, GitPath>>();

        deps
    }

    pub fn get_dependencies_to_update<'a>(
        &'a self,
        deps: &'a [&'a PkgInfo],
    ) -> Vec<&'a PkgInfo> {
        let mut to_update: Vec<&'a PkgInfo> = vec![];

        deps.iter().for_each(|contract| {
            if let Some(update) = self.get_updated_dependency(contract) {
                to_update.push(update);
            }
        });

        to_update
    }

    pub fn get_updated_dependency<'a>(
        &'a self,
        dep: &'a PkgInfo,
    ) -> Option<&'a PkgInfo> {
        // Fetch available versions by package name
        let versions = self.0.get(&dep.package.name).expect(
            format!(
                "Could not find Package Name {} in PkgRegistry",
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

    pub fn get_latest_protocol_version<'a>(
        &'a self,
        dep_name: &String,
    ) -> &'a Version {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).expect(
            format!("Could not find Package Name {} in PkgRegistry", dep_name)
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

    pub fn get_version_from_object_id(
        &self,
        object_id: &Address,
    ) -> Result<Version> {
        for (_, version_map) in self.0.iter() {
            let search_result = version_map.iter().find(|(_, contract)| {
                contract.contract_ref.object_id == *object_id
            });

            if let Some(search_result) = search_result {
                return Ok(*search_result.0);
            }
        }

        Err(anyhow!("Unable to find object ID in package map"))
    }

    // i.e. Sui or Originmate
    pub fn get_ext_dep_from_protocol(
        &self,
        ext_dep: String,
        version: &Version,
    ) -> GitPath {
        let protocol_versions =
            self.0.get(&String::from("NftProtocol")).expect(
                format!(
                    "Could not find Package Name {} in PkgRegistry",
                    &ext_dep
                )
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
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PkgInfo {
    pub package: Package,
    pub contract_ref: PkgPath,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, PkgPath>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PkgPath {
    pub path: GitPath,
    pub object_id: Address,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Package {
    pub name: String,
    pub version: Version,
    #[serde(rename(serialize = "published-at"))]
    pub published_at: Option<Address>,
}

impl Package {
    pub fn name_pascal(&self) -> String {
        self.name.as_str().to_case(Case::Pascal)
    }
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct GitPath {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}

impl GitPath {
    pub fn sanitize_subdir(&mut self) {
        if let Some(inner) = &mut self.subdir {
            if inner.is_empty() {
                self.subdir = None;
            }
        }
    }
}
