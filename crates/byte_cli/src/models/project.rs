use gutenberg::models::Address;
use serde::{Deserialize, Serialize};
use sui_sdk::types::base_types::ObjectID;

#[derive(Debug, Serialize, Deserialize)]
struct Project {
    name: String,
    project_owner: Address,
    package_id: ObjectID,
    publisher: ObjectID,
    upgrade_cap: ObjectID,
    admin_objects: AdminObjects,
    collection_objects: CollectionObjects,
}

#[derive(Debug, Serialize, Deserialize)]
struct AdminObjects {
    mint_caps: Vec<MintCap>,
    transfer_policy_caps: Vec<Cap>,
    withdraw_policy_caps: Vec<Cap>,
    borrow_policy_caps: Vec<Cap>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CollectionObjects {
    collection: ObjectID,
    royalty_bps: RoyaltyBPS,
    allowlists: Vec<ObjectID>,
    listing: ObjectID,
    warehouses: Vec<ObjectID>,
    venues: Vec<ObjectID>,
    transfer_policy: Vec<Policy>,
    withdraw_policy: Vec<Policy>,
    borrow_policy: Vec<Policy>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RoyaltyBPS {
    id: ObjectID,
    bps: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Policy {
    id: ObjectID,
    rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MintCap {
    id: ObjectID,
    supply: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Cap {
    id: ObjectID,
    objext_id: ObjectID,
}
