use std::{
    fmt::{self, Display},
    str::FromStr,
};

pub mod info;
pub mod pkg;
pub mod toml;
pub mod version;

pub const OB_PACKAGES: [&str; 12] = [
    "Pseudorandom",
    "Utils",
    "Critbit",
    "Permissions",
    "Request",
    "Kiosk",
    "Allowlist",
    "Authlist",
    "Launchpad",
    "LiquidityLayerV1",
    "LiquidityLayer",
    "NftProtocol",
];

pub enum Network {
    Mainnet,
    Testnet,
}

impl FromStr for Network {
    type Err = ();

    fn from_str(input: &str) -> Result<Network, Self::Err> {
        match input {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => Err(()),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        };

        f.write_str(string)
    }
}
