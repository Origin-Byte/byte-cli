use clap::{Parser, Subcommand};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;
use std::path::PathBuf;

pub use crate::consts::{
    DEFAULT_ASSETS_FOLDER, DEFAULT_CONFIG_FILENAME, DEFAULT_FOLDER,
};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Creates or adds confiiguration to JSON config file to be read by
    /// Gutenberg for the purpose of building the Move module
    ConfigCollection {
        /// Path to the directory with the config file
        #[clap(default_value = DEFAULT_FOLDER)]
        config_dir: String,
        #[clap(short, long, action)]
        complete: bool,
    },

    /// Creates or adds configuration to JSON config file to be read the asset
    /// deployer for the purpose of deploying assets, usually to an off-chain
    /// storage service
    ConfigUpload {
        /// Path to the directory with the config file
        #[clap(default_value = DEFAULT_FOLDER)]
        config_dir: String,
    },

    /// Combines `init-collection-config` and `init-upload-config in one single
    /// flow, hence make the UX seamless for the majority of use cases
    Config {
        /// Path to the directory with the config file
        #[clap(default_value = DEFAULT_FOLDER)]
        config_dir: String,
    },

    /// Deploys assets to a storage service
    DeployAssets {
        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS_FOLDER)]
        assets_dir: String,
    },

    /// Deploys NFT contract to Sui Blockchain
    DeployContract {
        /// Path to directory containing a Move package
        config: PathBuf,
        /// Gas budget for running module initializers
        #[clap(default_value_t = 60000)]
        gas_budget: usize,
        /// Sets the file for storing the state of user accounts
        client_config: Option<PathBuf>,
        /// Sets output directory
        output_dir: Option<PathBuf>,
    },

    /// Mints NFTs by calling the deployed contract
    MintNfts {
        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS_FOLDER)]
        config: PathBuf,
        /// Gas budget for running module initializers
        #[clap(default_value_t = 60000)]
        gas_budget: usize,
        warehouse_id: Option<String>,
    },
}

pub fn get_dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_style: Style::new(),
        checked_item_prefix: style("✔".to_string()).green().force_styling(true),
        unchecked_item_prefix: style("✔".to_string())
            .black()
            .force_styling(true),
        ..Default::default()
    }
}
