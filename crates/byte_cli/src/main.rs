pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod prelude;

use crate::prelude::*;
use endpoints::*;

use anyhow::Result;
use clap::Parser;
use console::style;
use std::path::PathBuf;

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
        Commands::InitCollectionConfig { output_file } => {
            let schema = &init_config::init_collection_config()?;
            init_config::write_config(
                schema,
                output_file
                    .unwrap_or_else(|| {
                        let mut path = PathBuf::new();
                        path.push(&schema.collection.name.to_lowercase());
                        path.set_extension("json");
                        path
                    })
                    .as_path(),
            )?;
        }
        Commands::InitUploadConfig { assets_dir: _ } => {}
        Commands::InitConfig { assets_dir: _ } => {}
        Commands::DeployAssets { assets_dir } => {
            deploy_assets::deploy_assets(assets_dir.as_str()).await?
        }
        Commands::DeployContract {
            config,
            output_dir,
            gas_budget,
            client_config,
        } => {
            let schema = deploy_contract::parse_config(config.as_path())?;
            let contract_dir = output_dir.unwrap_or_else(|| {
                let mut path = PathBuf::new();
                path.push(&schema.collection.name.to_lowercase());
                path
            });

            deploy_contract::generate_contract(
                &schema,
                contract_dir.as_path(),
            )?;

            deploy_contract::publish_contract(
                gas_budget,
                client_config.as_ref().map(PathBuf::as_path),
                &schema,
                contract_dir.as_path(),
            )?;
        }
        Commands::MintNfts { assets_dir: _ } => mint_nfts::mint_nfts().await,
    }

    Ok(())
}
