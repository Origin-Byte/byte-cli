use console::Emoji;

// CLI Select options
pub const FIELD_OPTIONS: [&str; 3] = ["display", "url", "attributes"];
pub const ROYALTY_OPTIONS: [&str; 3] = ["Proportional", "Constant", "None"];
pub const FEATURE_OPTIONS: [&str; 1] = ["tags"];
pub const SUPPLY_OPTIONS: [&str; 2] = ["Unlimited", "Limited"];
pub const MINTING_OPTIONS: [&str; 2] = ["launchpad", "airdrop"];
pub const MARKET_OPTIONS: [&str; 2] = ["Fixed price", "Dutch auction"];
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

// Smart contract invariants
pub const TX_SENDER_ADDRESS: &str = "sui::tx_context::sender(ctx)";

/// Default path for assets folder.
pub const DEFAULT_FOLDER: &str = "";
pub const DEFAULT_CONFIG_FILENAME: &str = "config";

pub const ROCKET_EMOJI: Emoji<'_, '_> = Emoji("🚀 ", "");

// Package commits
pub const SUI_PACKAGE_COMMIT: &str = "598f106ef5fbdfbe1b644236f0caf46c94f4d1b7";
pub const ORIGINMATE_PACKAGE_COMMIT: &str =
    "ef4e6b505f3fb1546944b770c78091b1ac47e392";
pub const PROTOCOL_PACKAGE_COMMIT: &str =
    "af32600e5fa7a0eccb2c87e1b8c18e4bb9129fe2";
