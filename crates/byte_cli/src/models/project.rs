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
    pub mint_caps: Vec<MintCap>,
    pub transfer_policy_caps: Vec<Cap>,
    pub withdraw_policy_caps: Vec<Cap>,
    pub borrow_policy_caps: Vec<Cap>,
}

impl AdminObjects {
    pub fn empty() -> AdminObjects {
        AdminObjects {
            mint_caps: vec![],
            transfer_policy_caps: vec![],
            withdraw_policy_caps: vec![],
            borrow_policy_caps: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionObjects {
    pub collection: Option<ObjectID>,
    pub royalty_bps: Option<RoyaltyBPS>,
    pub allowlists: Vec<ObjectID>,
    pub listing: Option<ObjectID>,
    pub warehouses: Vec<ObjectID>,
    pub venues: Vec<ObjectID>,
    pub transfer_policy: Vec<Policy>,
    pub withdraw_policy: Vec<Policy>,
    pub borrow_policy: Vec<Policy>,
}

impl CollectionObjects {
    pub fn empty() -> CollectionObjects {
        CollectionObjects {
            collection: None,
            royalty_bps: None,
            allowlists: vec![],
            listing: None,
            warehouses: vec![],
            venues: vec![],
            transfer_policy: vec![],
            withdraw_policy: vec![],
            borrow_policy: vec![],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoyaltyBPS {
    pub id: ObjectID,
    pub bps: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Policy {
    pub id: ObjectID,
    pub rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MintCap {
    pub id: ObjectID,
    // TODO: to add
    // pub supply: Option<u64>,
}

impl MintCap {
    pub fn new(id: ObjectID) -> Self {
        MintCap { id }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cap {
    pub id: ObjectID,
    pub object_id: ObjectID,
}
