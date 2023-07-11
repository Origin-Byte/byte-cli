use crate::models::collection::Supply;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

use super::Fields;

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Burn {
    Permissioned,
    Permissionless,
}

impl Display for Burn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Burn::Permissioned => "Permissioned",
            Burn::Permissionless => "Permissionless",
        };

        f.write_str(string)
    }
}

impl Burn {
    pub fn is_permissioned(&self) -> bool {
        matches!(self, Burn::Permissioned)
    }

    pub fn is_permissionless(&self) -> bool {
        matches!(self, Burn::Permissionless)
    }
}
