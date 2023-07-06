use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
// TODO: Add some kind of "optimistic" tracking that maintains supply tracking
// without requiring a mutable `Collection` parameter
pub enum Supply {
    Untracked,
    Tracked,
    Enforced(u64),
}

impl Default for Supply {
    /// Untracked is a reasonable default as it does not introduce any
    /// additional invariants that must be maintained by the user.
    fn default() -> Self {
        Supply::Untracked
    }
}

impl Supply {
    pub fn untracked() -> Self {
        Supply::Untracked
    }

    pub fn tracked() -> Self {
        Supply::Tracked
    }

    pub fn enforced(supply: u64) -> Self {
        Supply::Enforced(supply)
    }

    /// Whether supply needs to be tracked on the `Collection` level
    pub fn requires_collection(&self) -> bool {
        !matches!(self, Supply::Untracked)
    }
}
