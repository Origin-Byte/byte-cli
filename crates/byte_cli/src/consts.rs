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

/// Default path for assets folder.
pub const DEFAULT_FOLDER: &str = "";
pub const DEFAULT_CONFIG_FILENAME: &str = "config";

pub const ROCKET_EMOJI: Emoji<'_, '_> = Emoji("🚀 ", "");

// Package commits
pub const SUI_PACKAGE_COMMIT: &str = "ae1212baf8f0837e25926d941db3d26a61c1bea2";
pub const ORIGINMATE_PACKAGE_COMMIT: &str =
    "36e02283fa00451e8476a1bbc201af9a248396de";
pub const PROTOCOL_PACKAGE_COMMIT: &str =
    "b2dea4d1bee5608207d06d13ec0679a93d53962d";

pub const SUI_PACKAGE_COMMIT_TEST: &str =
    "38c31cdd0f2ae3963d1e06f00c766cd7f90a7be6";
pub const ORIGINMATE_PACKAGE_COMMIT_TEST: &str =
    "41d552066f55fb22609af24674ee06db842b8c52";
pub const PROTOCOL_PACKAGE_COMMIT_TEST: &str =
    "b3c85b735f047a17298d2640357bc75a67538890";
