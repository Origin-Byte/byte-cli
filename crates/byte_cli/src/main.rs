pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;
pub mod prelude;

use std::path::{Path, PathBuf};

use crate::prelude::*;
use byte_cli::utils::assert_no_unstable_features;
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
            project_dir,
            complete,
        } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("config.json");

            let mut schema = io::try_read_config(&file_path)?;

            schema =
                config_collection::init_collection_config(schema, complete)?;

            io::write_config(&schema, &file_path)?;
        }
        Commands::ConfigUpload { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("config.json");

            let mut schema = io::try_read_config(&file_path)?;

            schema = config_upload::init_upload_config(schema)?;

            io::write_config(&schema, &file_path)?;
        }
        Commands::Config { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("config.json");

            let mut schema = io::try_read_config(&file_path)?;

            schema = config_collection::init_collection_config(schema, false)?;
            schema = config_upload::init_upload_config(schema)?;

            io::write_config(&schema, &file_path)?;
        }
        Commands::DeployAssets { project_dir } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));

            let mut file_path = project_path.clone();
            file_path.push("config.json");

            let mut assets_path = project_path.clone();
            assets_path.push("assets/");
            let mut metadata_path = project_path.clone();
            metadata_path.push("metadata/");

            let schema = io::try_read_config(&file_path)?;

            // TODO: Remove this once all unstable features are completed
            assert_no_unstable_features(&schema)?;

            deploy_assets::deploy_assets(&schema, assets_path, metadata_path)
                .await?
        }
        Commands::GenerateContract { project_dir } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            let mut file_path = project_path.clone();
            file_path.push("config.json");

            let schema = deploy_contract::parse_config(file_path.as_path())?;

            // TODO: Remove this once all unstable features are completed
            assert_no_unstable_features(&schema)?;

            let mut contract_dir = project_path.clone();
            contract_dir.push("contract/");

            deploy_contract::generate_contract(
                &schema,
                contract_dir.as_path(),
            )?;
        }
        Commands::DeployContract {
            project_dir,
            gas_budget,
            skip_generation,
        } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            let mut file_path = project_path.clone();
            file_path.push("config.json");
            let mut state_path = project_path.clone();
            state_path.push("objects.json");

            let mut schema =
                deploy_contract::parse_config(file_path.as_path())?;

            // TODO: Remove this once all unstable features are completed
            assert_no_unstable_features(&schema)?;

            let mut contract_dir = project_path.clone();
            contract_dir.push("contract/");

            if skip_generation == false {
                deploy_contract::generate_contract(
                    &schema,
                    contract_dir.as_path(),
                )?;
            }

            let state = deploy_contract::publish_contract(
                &mut schema,
                gas_budget,
                contract_dir.as_path(),
            )
            .await?;

            // Updating with contract ID
            io::write_config(&schema, &file_path)?;
            io::write_collection_state(&state, &state_path)?;
        }
        Commands::MintNfts {
            project_dir: _,
            gas_budget: _,
            warehouse_id: _,
        } => {
            // TODO: Add back endpoint
            // let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            // let mut file_path = project_path.clone();
            // file_path.push("config.json");

            // let mut state_path = project_path.clone();
            // state_path.push("objects.json");

            // let mut metadata_path = project_path.clone();
            // metadata_path.push("metadata/");

            // let schema = deploy_contract::parse_config(file_path.as_path())?;

            // if schema.contract.is_none() {
            //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
            // }

            // if let Some(_contract) = &schema.contract {
            //     let mut state = CollectionState::try_read_config(&state_path)?;

            //     state = mint_nfts::mint_nfts(
            //         &schema,
            //         gas_budget,
            //         metadata_path,
            //         warehouse_id,
            //         state,
            //     )
            //     .await?;

            //     io::write_collection_state(&state, &state_path)?;
            // }
        }
        Commands::ConfigMarketplace { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("config.json");

            let mut schema = io::try_read_config(&file_path)?;

            // TODO: Remove this once all unstable features are completed
            assert_no_unstable_features(&schema)?;

            schema = config_marketplace::init_marketplace_config(schema)?;

            io::write_config(&schema, &file_path)?;
        }
        Commands::AddListingConfig {
            project_dir,
            skip_marketplace: _,
        } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("config.json");

            let mut schema = io::try_read_config(&file_path)?;
            // TODO: Remove this once all unstable features are completed
            assert_no_unstable_features(&schema)?;

            schema = add_listing::add_listing_config(schema)?;

            io::write_config(&schema, &file_path)?;
        }
    }

    Ok(())
}
