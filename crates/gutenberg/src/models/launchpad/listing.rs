use serde::{Deserialize, Serialize};

use super::{default_admin, market::Market};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct Listings(pub Vec<Listing>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Listing {
    #[serde(default = "default_admin")]
    pub admin: String,
    #[serde(default = "default_admin")]
    pub receiver: String,
    pub markets: Vec<Market>,
}

impl Listing {
    pub fn new(admin: String, receiver: String, markets: Vec<Market>) -> Self {
        Self {
            admin,
            receiver,
            markets,
        }
    }

    pub fn write_admin(&self) -> String {
        if self.admin == "sui::tx_context::sender(ctx)" {
            "sui::tx_context::sender(ctx)".to_string()
        } else {
            format!("@{}", self.admin)
        }
    }

    pub fn write_receiver(&self) -> String {
        if self.receiver == "sui::tx_context::sender(ctx)" {
            "sui::tx_context::sender(ctx)".to_string()
        } else {
            format!("@{}", self.receiver)
        }
    }

    pub fn write_init(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "
        let listing = nft_protocol::listing::new(
            {admin},
            {receiver},
            ctx,
        );

        let venue_id =
            nft_protocol::listing::create_venue(&mut listing, ctx);
",
            admin = self.write_admin(),
            receiver = self.write_receiver(),
        ));

        for market in self.markets.iter() {
            string.push_str(&market.init());
        }

        string.push_str(self.share());

        string
    }

    fn share(&self) -> &'static str {
        "
        transfer::share_object(listing);
"
    }
}
