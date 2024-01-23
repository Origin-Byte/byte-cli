use serde::{Deserialize, Serialize};
use std::fmt;
use sui_sdk::types::base_types::ObjectID;

/// Enum representing different types of objects in the system, each associated
/// with an `ObjectID`. This enumeration helps in differentiating between
/// various object types like Package, Collection, etc.
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

/// Implementation of the `Display` trait for `ObjectType`.
/// This allows for a human-readable representation of the object types,
/// focusing on their `ObjectID`.
impl fmt::Display for ObjectType {
    /// Formats an `ObjectType` as a string, displaying the contained
    /// `ObjectID`.
    ///
    /// # Arguments
    /// * `f` - The formatter.
    ///
    /// # Returns
    /// A result following the formatting operation, primarily displaying the
    /// `ObjectID`.
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
