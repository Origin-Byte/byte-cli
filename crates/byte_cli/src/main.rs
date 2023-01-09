pub mod cli;
pub mod consts;
pub mod prelude;

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
        Commands::InitCollectionConfig {} => {}
        Commands::InitUploadConfig { assets_dir } => {}
        Commands::InitConfig { assets_dir } => {}
        Commands::DeployAssets { assets_dir } => {}
        Commands::DeployContract {} => {}
        Commands::MintNfts { assets_dir } => {}
    }

    Ok(())
}
