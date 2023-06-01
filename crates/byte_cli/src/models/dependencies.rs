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
// pub nft_protocol: BTreeMap<Version, Contract>,
// pub liquidity_layer_v1: BTreeMap<Version, Contract>,
// pub launchpad: BTreeMap<Version, Contract>,
// pub liquidity_layer: BTreeMap<Version, Contract>,
// pub originmate: BTreeMap<Version, Contract>,
// pub authlist: BTreeMap<Version, Contract>,
// pub allowlist: BTreeMap<Version, Contract>,
// pub kiosk: BTreeMap<Version, Contract>,
// pub request: BTreeMap<Version, Contract>,
// pub permissions: BTreeMap<Version, Contract>,
// pub critbit: BTreeMap<Version, Contract>,
// pub utils: BTreeMap<Version, Contract>,
// pub pseudorandom: BTreeMap<Version, Contract>,

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
