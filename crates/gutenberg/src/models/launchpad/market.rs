use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
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

    pub fn init(&self, witness: &str) -> String {
        match self {
            Market::FixedPrice {
                token,
                price,
                is_whitelisted,
            } => format!(
                "
        nft_protocol::fixed_price::init_venue<{witness}, {token}>(
            &mut listing,
            inventory_id,
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
        nft_protocol::dutch_auction::init_venue<{witness}, {token}>(
            &mut listing,
            inventory_id,
            {is_whitelisted},
            {reserve_price},
            ctx,
        );
",
            ),
        }
    }
}
