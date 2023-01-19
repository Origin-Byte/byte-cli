//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use serde::{Deserialize, Serialize};
use std::{fmt::Write, str::FromStr};

use crate::prelude::GutenError;

fn default_admin() -> String {
    "tx_context::sender(ctx)".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Royalties {
    Proportional { bps: u64 },
    Constant { fee: u64 },
    None,
}

impl Royalties {
    pub fn new_from(
        input: &str,
        fee: Option<u64>,
    ) -> Result<Royalties, GutenError> {
        match input {
            "Proportional" => {
                let fee = fee.unwrap();
                Ok(Royalties::Proportional { bps: fee })
            }
            "Constant" => {
                let fee = fee.unwrap();
                Ok(Royalties::Constant { fee: fee })
            }
            "None" => Ok(Royalties::None),
            _ => Err(GutenError::UnsupportedRoyalty),
        }
    }

    pub fn write(&self) -> String {
        match self {
            Royalties::Proportional { bps } => {
                format!(
                    "royalty::add_proportional_royalty(
            &mut royalty,
            nft_protocol::royalty_strategy_bps::new({bps}),
        );",
                    bps = bps
                )
            }
            Royalties::Constant { fee } => {
                format!(
                    "royalty::add_constant_royalty(
            &mut royalty,
            nft_protocol::royalty_strategy_bps::new({fee}),
        );",
                    fee = fee
                )
            }
            Royalties::None => "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Tags(Vec<Tag>);

impl Tags {
    pub fn new(tags: &Vec<String>) -> Result<Self, GutenError> {
        let tags = tags
            .iter()
            .map(|string| {
                Tag::from_str(string).map_err(|_| GutenError::UnsupportedTag)
            })
            .collect::<Result<Vec<Tag>, GutenError>>()?;

        Ok(Tags(tags))
    }

    /// Generates Move code to push tags to a Move `vector` structure
    pub fn init(&self) -> String {
        let mut out = String::from("let tags = tags::empty(ctx);\n");

        for tag in self.0.iter() {
            out.write_fmt(format_args!(
                "        tags::add_tag(&mut tags, tags::{}());\n",
                tag.to_string()
            ))
            .unwrap();
        }

        out.push_str(
            "        tags::add_collection_tag_domain(&mut collection, &mut mint_cap, tags);"
        );

        out
    }

    pub fn push_tag(&mut self, tag_string: String) -> Result<(), GutenError> {
        let tag = Tag::from_str(tag_string.as_str())
            .map_err(|_| GutenError::UnsupportedTag)?;

        Ok(self.0.push(tag))
    }
}

/// Contains the market configurations of the marketplace
#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Listing {
    #[serde(default = "default_admin")]
    admin: String,
    #[serde(default = "default_admin")]
    receiver: String,
    markets: Vec<Market>,
}

impl Listing {
    pub fn new(admin: &str, receiver: &str, market: Market) -> Listing {
        Listing {
            admin: admin.to_string(),
            receiver: receiver.to_string(),
            markets: Vec::from([market]),
        }
    }

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
