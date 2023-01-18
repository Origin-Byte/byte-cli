pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod prelude;

use crate::endpoints::{deploy_assets, init_config};
use crate::prelude::*;
use anyhow::Result;
use clap::Parser;
use console::style;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {
            println!(
                "\n{}{}",
                consts::ROCKET_EMOJI,
                style("Process ran successfully.").green().bold().dim()
            );
        }
        Err(err) => {
            println!("\n{}", err,);
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::InitCollectionConfig {} => {
            init_config::init_collection_config()
        }
        Commands::InitUploadConfig { assets_dir: _ } => {}
        Commands::InitConfig { assets_dir: _ } => {}
        Commands::DeployAssets { assets_dir } => {
            deploy_assets::deploy_assets(assets_dir.as_str()).await?
        }
        Commands::DeployContract {} => {}
        Commands::MintNfts { assets_dir: _ } => {}
    }

    Ok(())
}
