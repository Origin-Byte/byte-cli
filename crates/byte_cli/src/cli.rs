use clap::{Parser, Subcommand};

pub use crate::consts::DEFAULT_ASSETS_FOLDER;

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
    InitCollectionConfig {},

    /// Creates or adds configuration to JSON config file to be read the asset
    /// deployer for the purpose of deploying assets, usually to an off-chain
    /// storage service
    InitUploadConfig {
        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS_FOLDER)]
        assets_dir: String,
    },

    /// Combines `InitCollectionConfig` and `InitUploadConfig in one single flow,
    /// hence make the UX seamless for the majority of use cases
    InitConfig {
        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS_FOLDER)]
        assets_dir: String,
    },

    /// Deploys assets to a storage service
    DeployAssets {
        /// Path to the directory with the assets
        #[clap(default_value = DEFAULT_ASSETS_FOLDER)]
        assets_dir: String,
    },

    /// Deploys NFT contract to Sui Blockchain
    DeployContract {},

    /// Mints NFTs by calling the deployed contract
    MintNfts {},
}
