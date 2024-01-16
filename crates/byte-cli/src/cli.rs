use clap::Parser;
use console::{style, Style};
use dialoguer::theme::ColorfulTheme;

/// Command Line Interface structure definition using clap.
// #[derive(Parser)]
// #[clap(author, version, about)]
// pub struct Cli {
//     #[clap(subcommand)]
//     pub command: Commands,
// }

/// Enum representing different command categories.
#[derive(Parser)]
pub enum Cli {
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

/// Enum representing specific Collection-related commands.
#[derive(Parser)]
pub enum CollectionCommands {
    #[clap(
        action,
        about = "Creates simple configuration file to be used for generating NFT collection contract"
    )]
    ConfigBasic {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },

    #[clap(
        action,
        about = "Creates a configuration file to be used for generating NFT collection contract"
    )]
    Config {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },
    #[clap(action, about = "Generates the NFT Collection smart contract")]
    Codegen {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },
}

/// Enum representing specific Image Upload-related commands.
#[derive(Parser)]
pub enum ImageCommands {
    #[clap(
        action,
        about = "Creates or adds configuration to JSON config file to be read by the asset deployer for the purpose of deploying assets, usually to an off-chain storage service"
    )]
    Config {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },
    #[clap(action, about = "Deploys assets to a storage service")]
    Upload {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },
}

/// Enum representing specific Sui Client-related commands.
#[derive(Parser)]
pub enum ClientCommands {
    #[clap(action, about = "Deploys NFT contract to Sui Blockchain")]
    PublishCollection {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
        #[clap(
            help = "Object ID of the Coin you would like to use to pay gas"
        )]
        gas_coin: Option<String>,
        /// Gas budget for running module initializers
        #[clap(help = "Gas limit for the transaction in MIST")]
        gas_budget: Option<usize>,
    },
    #[clap(
        action,
        about = "Creates an NFT Warehouse owned by the sender address"
    )]
    CreateWarehouse {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
        #[clap(
            help = "Object ID of the Coin you would like to use to pay gas"
        )]
        gas_coin: Option<String>,
        #[clap(
            help = "Gas limit for the transaction in MIST. Defaults to 5_000_000_000 MIST == 5 SUI"
        )]
        gas_budget: Option<usize>,
    },
    MintNfts {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            long,
            action,
            help = "The number of NFTs to mint. This number will be split in batches"
        )]
        amount: u64,
        #[clap(
            long,
            action,
            help = "The number of batches to divide the minting process into. So if you mint `1_000` as the amount and chose a `10` batches the minting process will be divided into 10 programmable transaction batches of 100 NFTs each."
        )]
        batches: u64,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
        /// Gas budget for minting an NFT
        #[clap(
            default_value_t = 60_000_000,
            help = "Gas limit for minting ONE NFT in MIST. Defaults to `60_000_000` MIST = 0.06 SUI"
        )]
        gas_budget: usize,
        #[clap(
            long,
            action,
            help = "Object ID of the Warehouse object that will hold the minted NFTs"
        )]
        warehouse_id: Option<String>,
        #[clap(
            long,
            action,
            help = "Object ID of the MintCap object of the Collection"
        )]
        mint_cap_id: Option<String>,
    },
}

/// Enum representing specific Coin Client-related commands.
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
        #[clap(
            default_value_t = 10_000_000,
            help = "Gas limit for the transaction in MIST. Defaults to `10_000_000` MIST = 0.01 SUI"
        )]
        gas_budget: usize,
        #[clap(
            short,
            long,
            action,
            help = "Object ID of the Coin you would like to use to pay gas"
        )]
        gas_coin: Option<String>,
    },

    #[clap(
        action,
        about = "Melts all SUI coins, except one, into a single coin"
    )]
    Melt {
        /// Gas budget
        #[clap(
            default_value_t = 50_000_000,
            help = "Gas limit for the transaction in MIST. Defaults to `50_000_000` MIST = 0.05 SUI"
        )]
        gas_budget: usize,
        #[clap(
            long,
            action,
            help = "Object ID of the Coin you would like to use to pay gas"
        )]
        gas_coin: String,
    },
}

/// Enum representing specific Move-related commands.
#[derive(Parser)]
pub enum MoveCommands {
    #[clap(action, about = "Updates OriginByte and Sui dependencies")]
    UpdateDependencies {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },

    #[clap(action, about = "Checks OriginByte and Sui dependencies")]
    CheckDependencies {
        #[clap(help = "The name of the NFT collection")]
        name: String,
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            short,
            long,
            action,
            help = "The path to the project directory (defaults to the Home directory)"
        )]
        project_dir: Option<String>,
    },

    #[clap(
        action,
        about = "Loads the dependencies for the Mainnet or Testnet environment"
    )]
    LoadEnv {
        #[clap(
            help = "Define the network environment: 'testnet' or 'mainnet'"
        )]
        network: String,
        #[clap(
            help = "The name of the NFT collection (In case the project directory is defaulted to the Home folder you have to provide the NFT project name)"
        )]
        name: Option<String>,
        #[clap(short, long, action)]
        project_dir: Option<String>,
    },
}

/// Creates and returns a dialoguer theme for consistent command-line interface
/// styling.
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
