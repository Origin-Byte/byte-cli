use serde::{Deserialize, Serialize};

/// Represents the policies regarding various requests in an NFT context.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestPolicies {
    /// Policy for transfer requests. `false` by default.
    #[serde(default)]
    transfer: bool,
    /// Policy for withdraw requests. `false` by default.
    #[serde(default)]
    withdraw: bool,
    /// Policy for borrow requests. `false` by default.
    #[serde(default)]
    borrow: bool,
}

impl Default for RequestPolicies {
    /// Creates a `RequestPolicies` instance with all policies set to `false`.
    /// This approach ensures safety by not enabling any policies by default.
    ///
    /// Future features requiring policies will be implemented considering safety.
    /// Not providing any request policies by default is reasonable as it does
    /// not have any implications that the creator might be forced to consider
    /// with regards to creating safe policies.
    ///
    /// Should other features require policies, they will be implemented in a
    /// safe way.
    fn default() -> Self {
        Self {
            transfer: false,
            withdraw: false,
            borrow: false,
        }
    }
}

// TODO: Move `RequestPolicies` to `NftData`
impl RequestPolicies {
    /// Constructs a new `RequestPolicies` instance with specified values for each policy.
    ///
    /// # Arguments
    /// * `transfer` - Boolean flag for the transfer policy.
    /// * `withdraw` - Boolean flag for the withdraw policy.
    /// * `borrow` - Boolean flag for the borrow policy.
    ///
    /// # Returns
    /// * `RequestPolicies` - A new instance of `RequestPolicies`.
    pub fn new(
        transfer: bool,
        withdraw: bool,
        borrow: bool,
    ) -> RequestPolicies {
        RequestPolicies {
            transfer,
            withdraw,
            borrow,
        }
    }

    /// Checks if the transfer policy is enabled.
    ///
    /// # Returns
    /// * `bool` - `true` if the transfer policy is enabled, else `false`.
    pub fn has_transfer(&self) -> bool {
        self.transfer
    }

    /// Checks if the withdraw policy is enabled.
    ///
    /// # Returns
    /// * `bool` - `true` if the withdraw policy is enabled, else `false`.
    pub fn has_withdraw(&self) -> bool {
        self.withdraw
    }

    /// Checks if the borrow policy is enabled.
    ///
    /// # Returns
    /// * `bool` - `true` if the borrow policy is enabled, else `false`.
    pub fn has_borrow(&self) -> bool {
        self.borrow
    }
}
