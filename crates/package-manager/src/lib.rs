pub mod info;
pub mod package;
pub mod toml;
pub mod version;

use anyhow::{anyhow, Result};
use git2::Repository;
use package::{Flavor, PackageRegistry};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    fs::File,
    path::PathBuf,
    str::FromStr,
};
use tempfile::TempDir;

/// Constant representing the available Origin Byte packages.
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
    "LiquidityLayer", // TODO: remove
    "NftProtocol",
];

/// Enum representing the different network environments.
#[derive(Deserialize, Serialize, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl FromStr for Network {
    type Err = ();

    /// Parses a string into a Network type.
    fn from_str(input: &str) -> Result<Network, Self::Err> {
        match input {
            "mainnet" => Ok(Network::Mainnet),
            "testnet" => Ok(Network::Testnet),
            _ => Err(()),
        }
    }
}

impl Display for Network {
    /// Implements formatting for displaying the Network type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Network::Mainnet => "mainnet",
            Network::Testnet => "testnet",
        };

        f.write_str(string)
    }
}

/// Retrieves package registries for both Mainnet and Testnet.
///
/// # Returns
/// Result containing a tuple of `PackageRegistry` for Mainnet and Testnet.
pub fn get_program_registries() -> Result<(PackageRegistry, PackageRegistry)> {
    let (temp_dir, mainnet_path, testnet_path) = get_pakage_registry_paths();

    let url = "https://github.com/Origin-Byte/program-registry";

    let repo = match Repository::clone(url, temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => return Err(anyhow!("failed to clone: {}", e)),
    };

    if repo.is_empty()? {
        return Err(anyhow!(
            "Something went wrong while accessing the Program Registry"
        ));
    }

    let mut main_registry: PackageRegistry =
        serde_json::from_reader(File::open(mainnet_path)?)?;
    let mut test_registry: PackageRegistry =
        serde_json::from_reader(File::open(testnet_path)?)?;

    main_registry.set_flavor(Flavor::Mainnet)?;
    test_registry.set_flavor(Flavor::Testnet)?;

    Ok((main_registry, test_registry))
}

/// Retrieves a program registry for a specified network.
///
/// # Arguments
/// * `network` - The network for which to retrieve the package registry.
///
/// # Returns
/// Result containing the `PackageRegistry` for the specified network.
pub fn get_program_registry(network: &Network) -> Result<PackageRegistry> {
    let (main_registry, test_registry) = get_program_registries()?;

    Ok(match network {
        Network::Mainnet => main_registry,
        Network::Testnet => test_registry,
    })
}

/// Generates temporary paths for main and test package registries.
///
/// # Returns
/// A tuple containing a `TempDir`, main registry `PathBuf`, and test registry
/// `PathBuf`.
pub fn get_pakage_registry_paths() -> (TempDir, PathBuf, PathBuf) {
    let temp_dir =
        TempDir::new().expect("Failed to create temporary directory");

    let mut registry_main_path = temp_dir.path().to_path_buf();
    registry_main_path.push("registry-main.json");

    let mut registry_test_path = temp_dir.path().to_path_buf();
    registry_test_path.push("registry-test.json");

    (temp_dir, registry_main_path, registry_test_path)
}
