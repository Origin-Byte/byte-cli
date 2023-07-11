pub mod listing;
pub mod market;
pub mod marketplace;

use super::address::Address;
use serde::{Deserialize, Serialize};

use self::{
    listing::{Listing, Listings},
    marketplace::Marketplace,
};

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Launchpad {
    /// In case a marketplace is creating the collection
    /// on behalf of the creator
    pub marketplace: Option<Marketplace>,
    pub listings: Listings,
}

impl Launchpad {
    pub fn set_listings(&mut self, listings: Listings) {
        self.listings = listings;
    }

    pub fn add_listing(&mut self, listing: Listing) {
        self.listings.0.push(listing);
    }
}
