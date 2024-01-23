use serde::{Deserialize, Serialize};

/// Struct representing different policies for minting tokens.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MintPolicies {
    /// Indicates whether the launchpad minting method is enabled.
    #[serde(default)]
    pub launchpad: bool,
    /// Indicates whether the airdrop minting method is enabled.
    #[serde(default)]
    pub airdrop: bool,
}

impl Default for MintPolicies {
    /// Provides a default set of minting policies.
    ///
    /// By default, sets `launchpad` to false and `airdrop` to true, as airdrop
    /// is applicable to all launch strategies, while launchpad is more specific
    /// to primary market contexts.
    ///
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
    /// Constructs a new `MintPolicies` instance with specified settings.
    ///
    /// # Arguments
    /// * `launchpad` - A boolean indicating if the launchpad method is enabled.
    /// * `airdrop` - A boolean indicating if the airdrop method is enabled.
    pub fn new(launchpad: bool, airdrop: bool) -> Self {
        Self { launchpad, airdrop }
    }

    /// Returns true if the launchpad minting method is enabled.
    pub fn has_launchpad(&self) -> bool {
        self.launchpad
    }

    /// Returns true if the airdrop minting method is enabled.
    pub fn has_airdrop(&self) -> bool {
        self.airdrop
    }
}
