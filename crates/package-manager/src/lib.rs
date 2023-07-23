pub mod info;
pub mod package;
pub mod toml;
pub mod version;

use anyhow::{anyhow, Result};
use git2::Repository;
use package::PackageRegistry;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs::File,
    path::PathBuf,
    str::FromStr,
};
use tempfile::TempDir;

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

#[derive(Deserialize, Serialize, PartialEq, Eq)]
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

pub fn get_program_registries() -> Result<(PackageRegistry, PackageRegistry)> {
    let (temp_dir, mainnet_path, testnet_path) = get_pakage_registry_paths();

    let url = "https://github.com/Origin-Byte/program-registry";

    let repo = match Repository::clone(url, temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => return Err(anyhow!("failed to clone: {}", e)),
    };

    if !repo.is_empty()? {
        println!("Fetched Program Registry successfully");
    } else {
        return Err(anyhow!(
            "Something went wrong while accessing the Program Registry"
        ));
    }

    let main_registry = serde_json::from_reader(File::open(mainnet_path)?)?;
    let test_registry = serde_json::from_reader(File::open(testnet_path)?)?;

    Ok((main_registry, test_registry))
}

pub fn get_program_registry(network: &Network) -> Result<PackageRegistry> {
    let (main_registry, test_registry) = get_program_registries()?;

    Ok(match network {
        Network::Mainnet => main_registry,
        Network::Testnet => test_registry,
    })
}

pub fn get_pakage_registry_paths() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir =
        TempDir::new().expect("Failed to create temporary directory");

    let mut registry_main_path = temp_dir.path().to_path_buf();
    registry_main_path.push("registry-main.json");

    let mut registry_test_path = temp_dir.path().to_path_buf();
    registry_test_path.push("registry-test.json");

    (temp_dir, registry_main_path, registry_test_path)
}
