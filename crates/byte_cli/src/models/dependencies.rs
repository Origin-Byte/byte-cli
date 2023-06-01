use anyhow::anyhow;
use gutenberg::models::Address;
use serde::{
    de::{self, Unexpected, Visitor},
    Deserialize, Deserializer,
};
use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    marker::PhantomData,
};

use crate::err::CliError;

use super::toml::{Dependency, Package, Version};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageMap(pub BTreeMap<String, BTreeMap<Version, Contract>>);

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Contract {
    pub package: Package,
    pub contract_ref: ContractRef,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, ContractRef>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContractRef {
    pub path: Dependency,
    pub object_id: Address,
}
