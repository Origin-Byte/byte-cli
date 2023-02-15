pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;
pub mod prelude;

use crate::prelude::*;
use endpoints::*;

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
        Commands::ConfigCollection {
            config_dir,
            complete,
        } => {
            let path = io::get_path_buf(config_dir.as_str());
            let mut schema = io::try_read_config(&path)?;

            schema =
                config_collection::init_collection_config(schema, complete)?;

            io::write_config(&schema, path.as_path())?;
        }
        Commands::ConfigUpload { config_dir } => {
            let path = io::get_path_buf(config_dir.as_str());
            let mut schema = io::try_read_config(&path)?;

            schema = config_upload::init_upload_config(schema)?;

            io::write_config(&schema, path.as_path())?;
        }
        Commands::Config { config_dir } => {
            let path = io::get_path_buf(config_dir.as_str());
            let mut schema = io::try_read_config(&path)?;

            schema = config_collection::init_collection_config(schema, false)?;
            schema = config_upload::init_upload_config(schema)?;

            io::write_config(&schema, path.as_path())?;
        }
        Commands::DeployAssets { assets_dir } => {
            deploy_assets::deploy_assets(assets_dir.as_str()).await?
        }
        Commands::DeployContract {
            project_dir,
            gas_budget,
        } => {
            let schema = deploy_contract::parse_config(project_dir.as_path())?;

            let mut contract_dir = project_dir.clone();
            contract_dir.push("contract");

            deploy_contract::generate_contract(
                &schema,
                contract_dir.as_path(),
            )?;

            deploy_contract::publish_contract(
                gas_budget,
                contract_dir.as_path(),
            )
            .await?;
        }
        Commands::MintNfts {
            config,
            gas_budget,
            warehouse_id,
        } => {
            // TODO: This method should not be in the deploy_contract
            let schema = deploy_contract::parse_config(config.as_path())?;

            if let Some(_contract) = &schema.contract {
                mint_nfts::mint_nfts(&schema, gas_budget, config, warehouse_id)
                    .await
            }
        }
    }

    Ok(())
}
