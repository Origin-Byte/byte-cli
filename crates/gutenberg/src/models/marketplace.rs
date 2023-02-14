//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use serde::{Deserialize, Serialize};

fn default_admin() -> String {
    "tx_context::sender(ctx)".to_string()
}

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize)]
pub struct Marketplace {
    #[serde(default = "default_admin")]
    pub admin: String,
    #[serde(default = "default_admin")]
    pub receiver: String,
}

impl Marketplace {
    pub fn init(&self) -> String {
        format!(
            "
        let marketplace = nft_protocol::marketplace::new(
            {admin},
            {receiver},
            nft_protocol::flat_fee::new(0, ctx),
            ctx,
        );
",
            admin = self.admin,
            receiver = self.receiver,
        )
    }

    pub fn share(&self) -> String {
        "
        transfer::share_object(marketplace);
"
        .to_string()
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Listings(pub Vec<Listing>);

#[derive(Debug, Serialize, Deserialize)]
pub struct Listing {
    #[serde(default = "default_admin")]
    pub admin: String,
    #[serde(default = "default_admin")]
    pub receiver: String,
    pub markets: Vec<Market>,
}

impl Listing {
    pub fn new(markets: Vec<Market>) -> Self {
        Self {
            admin: String::new(),
            receiver: String::new(),
            markets,
        }
    }

    pub fn write_admin(&self) -> String {
        if self.admin == "tx_context::sender(ctx)".to_string() {
            "tx_context::sender(ctx)".to_string()
        } else {
            format!("@{}", self.admin)
        }
    }

    pub fn write_receiver(&self) -> String {
        if self.receiver == "tx_context::sender(ctx)".to_string() {
            "tx_context::sender(ctx)".to_string()
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

#[derive(Debug, Serialize, Deserialize)]
pub enum Market {
    FixedPrice {
        /// Fully qualified fungible token in which price is denominated
        token: String,
        price: u64,
        is_whitelisted: bool,
    },
    DutchAuction {
        /// Fully qualified fungible token in which price is denominated
        token: String,
        reserve_price: u64,
        is_whitelisted: bool,
    },
}

impl Market {
    pub fn market_type(&self) -> &'static str {
        match self {
            Market::FixedPrice { .. } => "FixedPriceMarket",
            Market::DutchAuction { .. } => "DutchAuctionMarket",
        }
    }

    pub fn market_module(&self) -> &'static str {
        match self {
            Market::FixedPrice { .. } => "fixed_price",
            Market::DutchAuction { .. } => "dutch_auction",
        }
    }

    pub fn init(&self) -> String {
        match self {
            Market::FixedPrice {
                token,
                price,
                is_whitelisted,
            } => format!(
                "
        nft_protocol::fixed_price::create_market_on_listing<{token}>(
            &mut listing,
            venue_id,
            {is_whitelisted},
            {price},
            ctx,
        );
",
            ),
            Market::DutchAuction {
                token,
                reserve_price,
                is_whitelisted,
            } => format!(
                "
        nft_protocol::dutch_auction::create_market_on_listing<{token}>(
            &mut listing,
            venue_id,
            {is_whitelisted},
            {reserve_price},
            ctx,
        );
",
            ),
        }
    }
}
