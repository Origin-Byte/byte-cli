use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestPolicies {
    #[serde(default)]
    transfer: bool,
    #[serde(default)]
    withdraw: bool,
    #[serde(default)]
    borrow: bool,
}

impl Default for RequestPolicies {
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

    pub fn has_transfer(&self) -> bool {
        self.transfer
    }

    pub fn has_withdraw(&self) -> bool {
        self.withdraw
    }

    pub fn has_borrow(&self) -> bool {
        self.borrow
    }
}
