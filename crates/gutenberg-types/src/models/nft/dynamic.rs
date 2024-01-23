use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// A struct representing whether an NFT is dynamic or static.
/// It is marked for serialization and deserialization with `serde`.
#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(transparent)]
pub struct Dynamic(pub bool);

impl Default for Dynamic {
    /// Provides a default for `Dynamic`. By default, it is set to false,
    /// indicating a static NFT. This is chosen as the default as static NFTs
    /// are simpler and don't introduce additional complexities or potential
    /// security concerns that a dynamic NFT might have. Dynamic features can
    /// be added later if required.
    ///
    /// Static NFT by default is a reasonable default as it does not introduce
    /// any extra attack vectors that the creator might be forced to consider
    /// and dynamic features can always be added at a later date.
    fn default() -> Self {
        Self(false)
    }
}

impl Display for Dynamic {
    /// Implements the `Display` trait for `Dynamic`.
    /// This allows for a human-readable representation of the `Dynamic` type,
    /// displaying "Dynamic" if true and "Static" if false.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self.0 {
            true => "Dynamic",
            false => "Static",
        };

        f.write_str(string)
    }
}

impl Dynamic {
    /// Constructs a new `Dynamic` instance.
    /// This method allows for explicit creation of a `Dynamic` object.
    ///
    /// # Arguments
    /// * `dynamic` - A boolean value indicating whether the NFT is dynamic.
    pub fn new(dynamic: bool) -> Self {
        Self(dynamic)
    }

    /// Checks if the NFT is dynamic.
    /// Returns `true` if the NFT is dynamic, `false` otherwise.
    pub fn is_dynamic(&self) -> bool {
        self.0
    }
}
