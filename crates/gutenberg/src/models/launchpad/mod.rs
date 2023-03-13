pub mod listing;
pub mod market;
pub mod marketplace;

use serde::{Deserialize, Serialize};

use self::{
    listing::{Listing, Listings},
    marketplace::Marketplace,
};

fn default_admin() -> String {
    "tx_context::sender(ctx)".to_string()
}

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize, Default)]
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

    // TODO: To deprecate. The creation of listins will be done at runtime
    // in atomic transactions instead of being bundled up in the init funciton
    pub fn write_init_listings(&self, witness: &str) -> String {
        let code = self
            .listings
            .0
            .iter()
            .map(|listing| listing.write_init(witness))
            .collect::<Vec<_>>();

        code.join("\n")
    }
}
