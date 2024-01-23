use super::market::Market;
use super::Address;
use serde::{Deserialize, Serialize};

/// Struct representing a collection of Listings.
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Listings(pub Vec<Listing>);

/// Struct representing a single Listing.
///
/// A Listing includes information about the admin and receiver addresses,
/// as well as the associated markets.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    /// Address of the admin of the Listing.
    pub admin: Address,
    /// Address of the receiver for the Listing.
    pub receiver: Address,
    /// Vector of Market objects associated with the Listing.
    pub markets: Vec<Market>,
}

impl Listing {
    /// Constructs a new Listing.
    ///
    /// # Arguments
    /// * `admin` - Address of the admin.
    /// * `receiver` - Address of the receiver.
    /// * `markets` - Vector of associated markets.
    pub fn new(
        admin: Address,
        receiver: Address,
        markets: Vec<Market>,
    ) -> Self {
        Self {
            admin,
            receiver,
            markets,
        }
    }

    /// Provides a snippet for sharing the listing object.
    ///
    /// # Returns
    /// A string literal representing a code snippet for sharing the listing.
    pub fn share(&self) -> &'static str {
        "
        transfer::share_object(listing);
"
    }
}
