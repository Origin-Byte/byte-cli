//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use serde::Deserialize;
use std::str::FromStr;

fn default_admin() -> String {
    "tx_context::sender(ctx)".to_string()
}

pub enum Royalties {
    Proportional { bps: u64 },
    Constant { fee: u64 },
}

pub enum SupplyPolicy {
    Unlimited,
    Limited { max: u64 },
}

#[derive(Debug, Deserialize)]
pub enum Tag {
    Art,
    ProfilePicture,
    Collectible,
    GameAsset,
    TokenisedAsset,
    Ticker,
    DomainName,
    Music,
    Video,
    Ticket,
    License,
}

impl Tag {
    pub fn to_string(&self) -> String {
        let tag = match self {
            Tag::Art => "art",
            Tag::ProfilePicture => "profile_picture",
            Tag::Collectible => "collectible",
            Tag::GameAsset => "game_asset",
            Tag::TokenisedAsset => "tokenised_asset",
            Tag::Ticker => "ticker",
            Tag::DomainName => "domain_name",
            Tag::Music => "music",
            Tag::Video => "video",
            Tag::Ticket => "ticket",
            Tag::License => "license",
        };

        tag.to_string()
    }
}

impl FromStr for Tag {
    type Err = ();

    fn from_str(input: &str) -> Result<Tag, Self::Err> {
        match input {
            "Art" => Ok(Tag::Art),
            "ProfilePicture" => Ok(Tag::ProfilePicture),
            "Collectible" => Ok(Tag::Collectible),
            "GameAsset" => Ok(Tag::GameAsset),
            "TokenisedAsset" => Ok(Tag::TokenisedAsset),
            "Ticker" => Ok(Tag::Ticker),
            "DomainName" => Ok(Tag::DomainName),
            "Music" => Ok(Tag::Music),
            "Video" => Ok(Tag::Video),
            "Ticket" => Ok(Tag::Ticket),
            "License" => Ok(Tag::License),
            _ => Err(()),
        }
    }
}

/// Contains the market configurations of the marketplace
#[derive(Debug, Deserialize)]
pub struct Marketplace {
    #[serde(default = "default_admin")]
    admin: String,
    #[serde(default = "default_admin")]
    receiver: String,
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

#[derive(Debug, Deserialize)]
pub struct Listing {
    #[serde(default = "default_admin")]
    admin: String,
    #[serde(default = "default_admin")]
    receiver: String,
    markets: Vec<Market>,
}

impl Listing {
    pub fn init(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "
        let listing = nft_protocol::listing::new(
            {admin},
            {receiver},
            ctx,
        );
",
            admin = self.admin,
            receiver = self.receiver,
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

#[derive(Debug, Deserialize)]
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
        let inventory_id =
            nft_protocol::listing::create_inventory(&mut listing, ctx);

        nft_protocol::fixed_price::create_market_on_listing<{token}>(
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
        let inventory_id =
            nft_protocol::listing::create_inventory(&mut listing, ctx);

        nft_protocol::dutch_auction::create_market_on_listing<{token}>(
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
