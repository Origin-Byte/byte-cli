use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MintPolicies {
    #[serde(default)]
    pub launchpad: bool,
    #[serde(default)]
    pub airdrop: bool,
}

impl Default for MintPolicies {
    /// Most collections will need at least one kind of mint function, airdrop
    /// is a good candidate as it may be used for all launch strategies,
    /// whereas launchpad only makes sense in primary market contexts.
    fn default() -> Self {
        Self {
            launchpad: false,
            airdrop: true,
        }
    }
}

impl MintPolicies {
    pub fn new(launchpad: bool, airdrop: bool) -> Self {
        Self { launchpad, airdrop }
    }

    pub fn has_launchpad(&self) -> bool {
        self.launchpad
    }

    pub fn has_airdrop(&self) -> bool {
        self.airdrop
    }
}
