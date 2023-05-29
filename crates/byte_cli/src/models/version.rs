use gutenberg::models::Address;
use std::collections::{BTreeMap, HashMap};

use super::toml::{Dependency, Package};

pub struct Versions {
    pub nft_protocol: BTreeMap<Version, Contract>,
    pub liquidity_layer_v1: BTreeMap<Version, Contract>,
    pub launchpad: BTreeMap<Version, Contract>,
    pub launchpad_v2: BTreeMap<Version, Contract>,
    pub liquidity_layer: BTreeMap<Version, Contract>,
    pub originmate: BTreeMap<Version, Contract>,
    pub authlist: BTreeMap<Version, Contract>,
    pub allowlist: BTreeMap<Version, Contract>,
    pub kiosk: BTreeMap<Version, Contract>,
    pub request_extensions: BTreeMap<Version, Contract>,
    pub request: BTreeMap<Version, Contract>,
    pub permissions: BTreeMap<Version, Contract>,
    pub critbit: BTreeMap<Version, Contract>,
    pub utils: BTreeMap<Version, Contract>,
    pub pseudorandom: BTreeMap<Version, Contract>,
    pub sui: BTreeMap<Version, Contract>,
}

pub struct Contract {
    pub package: Package,
    pub contract_ref: ContractRef,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, ContractRef>,
}

pub struct ContractRef {
    pub path: Dependency,
    pub object_id: Address,
}

pub struct Version {
    // TODO: Make this equivalent to xx.yy.zz versioning
    pub version: u64,
}
