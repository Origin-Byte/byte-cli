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
    /// Creates simple configuration file to be used while generating contract
    ConfigSimple {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    /// Creates or adds configuration to JSON config file to be read while
    /// generating contract
    ConfigCollection {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    /// Creates or adds configuration to JSON config file to be read by the
    /// asset deployer for the purpose of deploying assets, usually to an
    /// off-chain storage service
    ConfigUpload {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    /// Deploys assets to a storage service
    UploadImages {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    /// Generates contract for later deployment
    // TODO: Add back
    // GenerateContract {
    //     name: String,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    // },

    /// Deploys NFT contract to Sui Blockchain
    // TODO: Add back
    // DeployContract {
    //     name: String,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    //     /// Gas budget for running module initializers
    //     #[clap(default_value_t = 600_000_000)]
    //     gas_budget: usize,
    // },

    /// Mints NFTs by calling the deployed contract
    // TODO: Add back
    // CreateWarehouse {
    //     name: String,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    //     /// Gas budget for minting an NFT
    //     #[clap(default_value_t = 50_000_000_000)]
    //     gas_budget: usize,
    // },
    // TODO: Add back
    // MintNfts {
    //     name: String,
    //     // #[clap(long, action)]
    //     // amount: Option<u64>,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    //     /// Gas budget for minting an NFT
    //     #[clap(default_value_t = 50_000_000_000)]
    //     gas_budget: usize,
    //     #[clap(long, action)]
    //     warehouse_id: Option<String>,
    //     #[clap(long, action)]
    //     mint_cap_id: Option<String>,
    // },
    ListCoins {},

    SplitCoin {
        #[clap(short, long, action)]
        coin_id: String,
        count: u64,
        #[clap(short, long, action)]
        amount: Option<u64>,
        #[clap(default_value_t = 10_000_000)]
        gas_budget: usize,
        #[clap(short, long, action)]
        gas_id: Option<String>,
    },
    CombineCoins {
        /// Gas budget
        #[clap(default_value_t = 50_000_000)]
        gas_budget: usize,
        #[clap(long, action)]
        gas_id: String,
    },
    CheckDependencies {
        name: String,
        network: Option<String>,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
    UseEnv {
        network: String,
        name: Option<String>,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
    // TODO: Add back
    // ParallelMint {
    //     name: String,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    //     /// Gas budget for minting an NFT
    //     #[clap(default_value_t = 18_000_000_000)]
    //     gas_budget: usize,
    //     main_gas_id: String,
    //     minor_gas_id: String,
    // },
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
