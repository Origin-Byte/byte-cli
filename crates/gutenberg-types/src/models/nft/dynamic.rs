use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(transparent)]
pub struct Dynamic(pub bool);

impl Default for Dynamic {
    /// Static NFT by default is a reasonable default as it does not introduce
    /// any extra attack vectors that the creator might be forced to consider
    /// and dynamic features can always be added at a later date.
    fn default() -> Self {
        Self(false)
    }
}

impl Display for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self.0 {
            true => "Dynamic",
            false => "Static",
        };

        f.write_str(string)
    }
}

impl Dynamic {
    pub fn new(dynamic: bool) -> Self {
        Self(dynamic)
    }

    pub fn is_dynamic(&self) -> bool {
        self.0
    }
}
