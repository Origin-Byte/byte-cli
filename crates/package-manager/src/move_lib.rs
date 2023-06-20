use gutenberg::models::Address;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::version::Version;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageMap(pub BTreeMap<String, BTreeMap<Version, MoveLib>>);

impl PackageMap {
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
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct MoveLib {
    pub package: Package,
    pub contract_ref: LibSpecs,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, LibSpecs>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct LibSpecs {
    pub path: Dependency,
    pub object_id: Address,
}

#[derive(Deserialize, Debug, Serialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Package {
    pub name: String,
    pub version: Version,
    #[serde(rename(serialize = "published-at"))]
    pub published_at: Option<Address>,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Dependency {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}

impl Dependency {
    pub fn sanitize_subdir(&mut self) {
        if let Some(inner) = &mut self.subdir {
            if inner.is_empty() {
                self.subdir = None;
            }
        }
    }
}
