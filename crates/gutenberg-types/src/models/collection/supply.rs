use serde::{Deserialize, Serialize};

/// An enum representing different supply tracking options for an NFT collection.
/// It provides flexibility in how the supply of NFTs is managed.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
// TODO: Add some kind of "optimistic" tracking that maintains supply tracking
// without requiring a mutable `Collection` parameter
pub enum Supply {
    /// Variant for untracked supply.
    /// This option allows for NFTs to be created without explicit supply tracking.
    Untracked,
    /// Variant for tracked supply.
    /// In this case, the supply of NFTs is explicitly tracked.
    Tracked,
    /// Variant for enforced supply with a specified limit.
    /// This option enforces a specific supply limit for the NFT collection.
    Enforced(u64),
}

impl Default for Supply {
    /// Provides a default supply tracking option.
    /// By default, it is set to `Untracked`, which does not impose any additional
    /// invariants that must be maintained by the user.
    ///
    /// Untracked is a reasonable default as it does not introduce any
    /// additional invariants that must be maintained by the user.
    fn default() -> Self {
        Supply::Untracked
    }
}

impl Supply {
    /// Creates a `Supply` instance with untracked supply.
    pub fn untracked() -> Self {
        Supply::Untracked
    }

    /// Creates a `Supply` instance with tracked supply.
    pub fn tracked() -> Self {
        Supply::Tracked
    }

    /// Creates a `Supply` instance with enforced supply and a specified limit.
    ///
    /// # Arguments
    /// * `supply` - The enforced supply limit for the NFT collection.
    pub fn enforced(supply: u64) -> Self {
        Supply::Enforced(supply)
    }

    /// Determines whether supply tracking on the `Collection` level is required.
    /// This method checks if the supply tracking option is not `Untracked`.
    pub fn requires_collection(&self) -> bool {
        !matches!(self, Supply::Untracked)
    }
}
