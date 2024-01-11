use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// Enum representing different types of Burn policies.
#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Burn {
    Permissioned,
    Permissionless,
}

impl Display for Burn {
    /// Implements formatting for displaying the Burn type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Burn::Permissioned => "Permissioned",
            Burn::Permissionless => "Permissionless",
        };

        f.write_str(string)
    }
}

impl Burn {
    /// Returns true if the Burn policy is Permissioned.
    pub fn is_permissioned(&self) -> bool {
        matches!(self, Burn::Permissioned)
    }

    /// Returns true if the Burn policy is Permissionless.
    pub fn is_permissionless(&self) -> bool {
        matches!(self, Burn::Permissionless)
    }
}
