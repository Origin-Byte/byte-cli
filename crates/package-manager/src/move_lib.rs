use gutenberg::models::Address;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::version::Version;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageMap(pub BTreeMap<String, BTreeMap<Version, MoveLib>>);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MoveLib {
    pub package: Package,
    pub contract_ref: LibSpecs,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, LibSpecs>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LibSpecs {
    pub path: Dependency,
    pub object_id: Address,
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
