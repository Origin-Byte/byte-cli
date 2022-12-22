//! Module containing enums that facilitate the generation of Move
//! code. Fields in the yaml file, such as `NftType`, are represented via a
//! String but should match to a value in a given Enum. Such Enums represent
//! the type of NFTs available or the type of Markets available on our
//! OriginByte protocol.
use serde::Deserialize;

fn default_admin() -> String {
    "tx_context::sender(ctx)".to_string()
}

/// Enum representing the NFT types currently available in the protocol
#[derive(Debug, Deserialize)]
pub enum NftType {
    // TODO: Need to add support for Soulbound
    Classic,
    // TODO: To be added back
    // Collectible,
    // CNft,
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

impl NftType {
    /// Writes Move code for an entry function meant to be called by
    /// the Creators to mint NFTs. Depending on the NFTtype the function
    /// parameters change, therefore pattern match the NFT type.
    pub fn mint_func(&self, witness: &str) -> Box<str> {
        let func = match self {
            NftType::Classic => format!(
                "public entry fun mint_nft(
                    name: String,
                    description: String,
                    url: vector<u8>,
                    attribute_keys: vector<String>,
                    attribute_values: vector<String>,
                    mint_cap: &mut MintCap<{witness}>,
                    slot: &mut Slot,
                    market_id: ID,
                    ctx: &mut TxContext,
                ) {{
                    let nft = nft::new<{witness}>(tx_context::sender(ctx), ctx);

                    collection::increment_supply(mint_cap, 1);

                    display::add_display_domain(
                        &mut nft,
                        name,
                        description,
                        ctx,
                    );

                    display::add_url_domain(
                        &mut nft,
                        url::new_unsafe_from_bytes(url),
                        ctx,
                    );

                    display::add_attributes_domain_from_vec(
                        &mut nft,
                        attribute_keys,
                        attribute_values,
                        ctx,
                    );

                    slot::add_nft(slot, market_id, nft, ctx);
                }}",
                witness = witness,
            ),
        };
        func.into_boxed_str()
    }
}

/// Contains the market configurations of the launchpad
#[derive(Debug, Deserialize)]
pub struct Launchpad {
    #[serde(default = "default_admin")]
    admin: String,
    #[serde(default = "default_admin")]
    receiver: String,
}

impl Launchpad {
    pub fn init(&self) -> String {
        format!(
            "
        let launchpad = nft_protocol::launchpad::new(
            {admin},
            {receiver},
            false,
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
        transfer::share_object(launchpad);
"
        .to_string()
    }
}

#[derive(Debug, Deserialize)]
pub struct Slot {
    #[serde(default = "default_admin")]
    admin: String,
    #[serde(default = "default_admin")]
    receiver: String,
    markets: Vec<Market>,
}

impl Slot {
    pub fn init(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "
        let slot = nft_protocol::slot::new(
            &launchpad,
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
        transfer::share_object(slot);
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
        nft_protocol::fixed_price::init_market<{token}>(
            &mut slot,
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
        nft_protocol::dutch_auction::init_market<{token}>(
            &mut slot,
            {is_whitelisted},
            {reserve_price},
            ctx,
        );
",
            ),
        }
    }
}
