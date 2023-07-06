use super::market::Market;
use super::Address;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Listings(pub Vec<Listing>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    pub admin: Address,
    pub receiver: Address,
    pub markets: Vec<Market>,
}

impl Listing {
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

    pub fn share(&self) -> &'static str {
        "
        transfer::share_object(listing);
"
    }
}
