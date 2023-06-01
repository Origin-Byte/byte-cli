use serde::{Deserialize, Serialize};
use sui_sdk::types::base_types::{ObjectID, SuiAddress};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub project_owner: SuiAddress,
    pub package_id: Option<ObjectID>,
    pub publisher: Option<ObjectID>,
    pub upgrade_cap: Option<ObjectID>,
    pub admin_objects: Option<AdminObjects>,
    pub collection_objects: Option<CollectionObjects>,
}

impl Project {
    pub fn new(name: String, project_owner: SuiAddress) -> Project {
        Project {
            name,
            project_owner,
            package_id: None,
            publisher: None,
            upgrade_cap: None,
            admin_objects: None,
            collection_objects: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminObjects {
    mint_caps: Vec<MintCap>,
    transfer_policy_caps: Vec<Cap>,
    withdraw_policy_caps: Vec<Cap>,
    borrow_policy_caps: Vec<Cap>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionObjects {
    collection: ObjectID,
    royalty_bps: RoyaltyBPS,
    allowlists: Vec<ObjectID>,
    listing: Option<ObjectID>,
    warehouses: Vec<ObjectID>,
    venues: Vec<ObjectID>,
    transfer_policy: Vec<Policy>,
    withdraw_policy: Vec<Policy>,
    borrow_policy: Vec<Policy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoyaltyBPS {
    id: ObjectID,
    bps: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    id: ObjectID,
    rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintCap {
    id: ObjectID,
    supply: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cap {
    id: ObjectID,
    objext_id: ObjectID,
}
