use console::Emoji;
use package_manager::Network;

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
pub const LAUNCHPAD_ID_MAIN: &str =
    "0x5cf2b8379d7471113852dbf343c14f933ccaca527bbe37b42724b5dde4738830";
pub const LAUNCHPAD_ID_TEST: &str =
    "0xf4feb74af60c3baa3cb3c50332edf3b0c2e9e00d353120c41b86182aee342db8";
pub const MAX_SYMBOL_LENGTH: u64 = 5;
pub const BPS_100_PERCENT: u64 = 10_000;
pub const DEFAULT_GAS_BUDGET: u64 = 50_000_000_000;

pub const KIWI_EMOJI: Emoji<'_, '_> = Emoji("🥝 ", "");

pub fn get_launchpad_id(network: &Network) -> &str {
    match network {
        Network::Mainnet => LAUNCHPAD_ID_MAIN,
        Network::Testnet => LAUNCHPAD_ID_TEST,
    }
}
