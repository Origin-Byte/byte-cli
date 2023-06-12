use serde::{Deserialize, Serialize};
use std::fmt;
use sui_sdk::types::base_types::ObjectID;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CollectionState {
    pub contract: Option<ObjectType>,
    pub collection: Option<ObjectType>,
    pub mint_cap: Option<ObjectType>,
    pub warehouses: Vec<ObjectType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ObjectType {
    Package(ObjectID),
    Collection(ObjectID),
    MintCap(ObjectID),
    Warehouse(ObjectID),
    BpsRoyaltyStrategy(ObjectID),
    PolicyCap(ObjectID),
    Policy(ObjectID),
    TransferPolicy(ObjectID),
    TransferPolicyCap(ObjectID),
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Transparently pass through ObjectID
        match self {
            ObjectType::Package(id)
            | ObjectType::Collection(id)
            | ObjectType::MintCap(id)
            | ObjectType::Warehouse(id)
            | ObjectType::BpsRoyaltyStrategy(id)
            | ObjectType::PolicyCap(id)
            | ObjectType::Policy(id)
            | ObjectType::TransferPolicy(id)
            | ObjectType::TransferPolicyCap(id) => fmt::Display::fmt(id, f),
        }
    }
}
