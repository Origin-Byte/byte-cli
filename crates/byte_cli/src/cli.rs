use clap::{Parser, Subcommand};
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    NewSimple {
        name: String,
        supply: usize,
        royalty_bps: usize,
        project_dir: Option<String>,
    },

    /// Creates or adds confiiguration to JSON config file to be read by
    /// Gutenberg for the purpose of building the Move module
    ConfigCollection {
        name: String,
        project_dir: Option<String>,
        #[clap(short, long, action)]
        complete: bool,
    },

    /// Creates or adds configuration to JSON config file to be read the asset
    /// deployer for the purpose of deploying assets, usually to an off-chain
    /// storage service
    ConfigUpload {
        name: String,
        project_dir: Option<String>,
    },

    /// Combines `init-collection-config` and `init-upload-config in one single
    /// flow, hence make the UX seamless for the majority of use cases
    Config {
        name: String,
        project_dir: Option<String>,
    },

    /// Deploys assets to a storage service
    DeployAssets {
        name: String,
        project_dir: Option<String>,
    },

    /// Deploys NFT contract to Sui Blockchain
    GenerateContract {
        name: String,
        project_dir: Option<String>,
        version: Option<String>,
    },

    /// Deploys NFT contract to Sui Blockchain
    DeployContract {
        name: String,
        project_dir: Option<String>,
        /// Gas budget for running module initializers
        #[clap(default_value_t = 600_000_000)]
        gas_budget: usize,
        #[clap(short, long, action)]
        skip_generation: bool,
        version: Option<String>,
    },

    /// Mints NFTs by calling the deployed contract
    MintNfts {
        name: String,
        project_dir: Option<String>,
        /// Gas budget for minting an NFT
        #[clap(default_value_t = 50_000_000_000)]
        gas_budget: usize,
        #[clap(long, action)]
        warehouse_id: Option<String>,
    },
    ParallelMint {
        name: String,
        project_dir: Option<String>,
        /// Gas budget for minting an NFT
        #[clap(default_value_t = 18_000_000_000)]
        gas_budget: usize,
        #[clap(long, action)]
        warehouse_id: Option<String>,
    },
    SplitCoin {
        /// Gas budget
        amount: u64,
        count: u64,
        #[clap(default_value_t = 50_000_000)]
        gas_budget: usize,
    },
    CombineCoins {
        /// Gas budget
        #[clap(default_value_t = 50_000_000)]
        gas_budget: usize,
    },
    CheckDependencies {
        name: String,
        project_dir: Option<String>,
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
