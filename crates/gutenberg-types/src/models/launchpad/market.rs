use serde::{Deserialize, Serialize};

/// An enum representing different types of markets for trading assets.
/// It supports fixed price sales and Dutch auctions.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum Market {
    /// Variant for a market with a fixed price.
    /// - `token`: Specifies the fully qualified fungible token in which the price is denominated.
    /// - `price`: The fixed price for the asset.
    /// - `is_whitelisted`: Indicates whether the market is whitelisted.
    FixedPrice {
        /// Fully qualified fungible token in which price is denominated
        token: String,
        price: u64,
        is_whitelisted: bool,
    },
    /// Variant for a Dutch auction market.
    /// - `token`: Specifies the fully qualified fungible token in which the reserve price is denominated.
    /// - `reserve_price`: The minimum price at which the asset can be sold in the auction.
    /// - `is_whitelisted`: Indicates whether the market is whitelisted.
    DutchAuction {
        /// Fully qualified fungible token in which price is denominated
        token: String,
        reserve_price: u64,
        is_whitelisted: bool,
    },
}

impl Market {
    /// Returns a string literal representing the type of market.
    /// This method differentiates between `FixedPrice` and `DutchAuction` markets,
    /// returning a string that represents their respective types.
    pub fn market_type(&self) -> &'static str {
        match self {
            Market::FixedPrice { .. } => "FixedPriceMarket",
            Market::DutchAuction { .. } => "DutchAuctionMarket",
        }
    }

    /// Returns the name of the module associated with the market type.
    /// This method provides the module name corresponding to the market type,
    /// which is used in Move code generation and other logic related to market operations.
    pub fn market_module(&self) -> &'static str {
        match self {
            Market::FixedPrice { .. } => "fixed_price",
            Market::DutchAuction { .. } => "dutch_auction",
        }
    }
}
