use console::Emoji;

// CLI Select options
pub const ROYALTY_OPTIONS: [&str; 2] =
    ["Percentage of trade price (in Basis Points)", "None"];
pub const FEATURE_OPTIONS: [&str; 3] = [
    "Tradeable Traits",
    "Immediate Secondary Market Trading",
    "NFT Burning",
];
pub const MARKET_OPTIONS: [&str; 2] =
    ["Fixed price sale", "Dutch auction sale"];
pub const TAG_OPTIONS: [&str; 11] = [
    "Art",
    "ProfilePicture",
    "Collectible",
    "GameAsset",
    "TokenisedAsset",
    "Ticker",
    "DomainName",
    "Music",
    "Video",
    "Ticket",
    "License",
];

pub const MINTING_OPTIONS_: [&str; 2] = ["launchpad", "airdrop"];
pub const MARKET_OPTIONS_: [&str; 2] = ["Fixed price", "Dutch auction"];

// Misc
pub const TX_SENDER_ADDRESS: &str = "tx_context::sender(ctx)";
pub const MAX_SYMBOL_LENGTH: u64 = 5;
pub const BPS_100_PERCENT: u64 = 10_000;

pub const ROCKET_EMOJI: Emoji<'_, '_> = Emoji("🚀 ", "");
