use clap::Parser;
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Parser)]
pub enum Commands {
    #[clap(about = "Account-related commands")]
    Account {
        #[clap(subcommand)]
        cmd: AccountCommands,
    },

    #[clap(about = "NFT Collection-related commands")]
    Collection {
        #[clap(subcommand)]
        cmd: CollectionCommands,
    },

    #[clap(about = "Image Upload-related commands")]
    Images {
        #[clap(subcommand)]
        cmd: ImageCommands,
    },

    #[clap(about = "Sui Client-related commands")]
    Client {
        #[clap(subcommand)]
        cmd: ClientCommands,
    },

    #[clap(about = "Coin Client-related commands")]
    Coin {
        #[clap(subcommand)]
        cmd: CoinCommands,
    },

    #[clap(about = "Move-related commands")]
    Move {
        #[clap(subcommand)]
        cmd: MoveCommands,
    },
}

#[derive(Parser)]
pub enum AccountCommands {
    #[clap(action, about = "Create a SuiPlay account")]
    Create {
        #[clap(short, long)]
        root_dir: Option<String>,
    },

    #[clap(about = "Link a SuiPlay account to your local configuration")]
    Link {
        #[clap(short, long, action)]
        root_dir: Option<String>,
    },

    #[clap(about = "Switch the default SuiPlay account")]
    Switch {
        email: String,
        #[clap(short, long, action)]
        root_dir: Option<String>,
    },
}

#[derive(Parser)]
pub enum CollectionCommands {
    #[clap(
        action,
        about = "Creates simple configuration file to be used for generating NFT collection contract"
    )]
    ConfigBasic {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    #[clap(
        action,
        about = "Creates a configuration file to be used for generating NFT collection contract"
    )]
    Config {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
    // Generates contract for later deployment
    // TODO: Add back
    // GenerateContract {
    //     name: String,
    //     #[clap(short, long, action)]
    //     project_dir: Option<String>,
    // },
}

#[derive(Parser)]
pub enum ImageCommands {
    #[clap(
        action,
        about = "Creates or adds configuration to JSON config file to be read by the asset deployer for the purpose of deploying assets, usually to an off-chain storage service"
    )]
    Config {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
    #[clap(action, about = "Deploys assets to a storage service")]
    Upload {
        name: String,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
}

#[derive(Parser)]
pub enum ClientCommands {
    #[clap(action, about = "Deploys NFT contract to Sui Blockchain")]
    PublishNftCollection {
        name: String,
        network: Option<String>,
        #[clap(short, long, action)]
        project_dir: Option<String>,
        /// Gas budget for running module initializers
        #[clap(default_value_t = 600_000_000)]
        gas_budget: usize,
    },
    // Mints NFTs by calling the deployed contract
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

#[derive(Parser)]
pub enum CoinCommands {
    #[clap(action, about = "Lists all SUI coins")]
    List {},

    #[clap(action, about = "Splits a SUI coin into equal chunks")]
    Split {
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

    #[clap(
        action,
        about = "Melts all SUI coins, except one, into a single coin"
    )]
    Melt {
        /// Gas budget
        #[clap(default_value_t = 50_000_000)]
        gas_budget: usize,
        #[clap(long, action)]
        gas_id: String,
        // up_to
    },
}

#[derive(Parser)]
pub enum MoveCommands {
    #[clap(action, about = "Checks OriginByte and Sui dependencies")]
    CheckDependencies {
        name: String,
        network: Option<String>,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },

    #[clap(
        action,
        about = "Loads the dependencies for the Mainnet or Testnet environment"
    )]
    LoadEnv {
        network: String,
        name: Option<String>,
        #[clap(short, long, action)]
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
