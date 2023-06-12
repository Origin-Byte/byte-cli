pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;
pub mod prelude;

use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::prelude::*;
use byte_cli::{
    consts::{CONFIG_FILENAME, PROJECT_FILENAME},
    io::{LocalRead, LocalWrite},
};
use endpoints::*;

use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;

use git2::Repository;
use gutenberg::schema::SchemaBuilder;
use package_manager::{
    info::BuildInfo,
    move_lib::PackageMap,
    toml::{self as move_toml, MoveToml},
};
use rust_sdk::coin;
use std::io::Write;
use tempfile::TempDir;
use uploader::writer::Storage;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {
            println!(
                "\n{}{}",
                consts::ROCKET_EMOJI,
                style("Process ran successfully.")
                    .green()
                    .bold()
                    .on_bright()
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
        Commands::NewSimple {
            name,
            supply,
            royalty_bps,
            project_dir,
        } => {
            let mut project_path =
                PathBuf::from(Path::new(project_dir.as_str()));

            let mut schema_path = project_path.clone();
            schema_path.push(CONFIG_FILENAME);
            project_path.push(PROJECT_FILENAME);

            let (schema, project) =
                new_simple::init_schema(&name, supply, royalty_bps).await?;

            schema.write(&schema_path)?;
            project.write(&project_path)?;
        }
        Commands::ConfigCollection {
            project_dir,
            complete,
        } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push(CONFIG_FILENAME);

            let mut builder = SchemaBuilder::read(&file_path)?;

            builder =
                config_collection::init_collection_config(builder, complete)?;

            builder.write(&file_path)?;
        }
        Commands::ConfigUpload { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push(CONFIG_FILENAME);

            let uploader = config_upload::init_upload_config()?;

            uploader.write(&file_path)?;
        }
        Commands::Config { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push(CONFIG_FILENAME);

            let mut builder = SchemaBuilder::read(&file_path)?;

            builder =
                config_collection::init_collection_config(builder, false)?;
            let uploader = config_upload::init_upload_config()?;

            builder.write(&file_path)?;
            uploader.write(&file_path)?;
        }
        Commands::DeployAssets { project_dir } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));

            let mut file_path = project_path.clone();
            // TODO: Incorrect since we separated files
            file_path.push(CONFIG_FILENAME);

            let mut assets_path = project_path.clone();
            assets_path.push("assets/");
            let mut metadata_path = project_path.clone();
            metadata_path.push("metadata/");

            let uploader = Storage::read(&file_path)?;

            deploy_assets::deploy_assets(&uploader, assets_path, metadata_path)
                .await?
        }
        Commands::GenerateContract { project_dir } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            let mut file_path = project_path.clone();
            file_path.push("config.json");

            let schema = deploy_contract::parse_config(file_path.as_path())?;

            let mut contract_dir = project_path;
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

            let schema = deploy_contract::parse_config(file_path.as_path())?;

            let mut contract_dir = project_path.clone();
            contract_dir.push("contract/");

            if !skip_generation {
                deploy_contract::generate_contract(
                    &schema,
                    contract_dir.as_path(),
                )?;
            }

            let state = deploy_contract::publish_contract(
                gas_budget,
                &PathBuf::from(contract_dir.as_path()),
            )
            .await?;

            state.write(&state_path)?;
        }
        Commands::MintNfts {
            project_dir,
            gas_budget,
            warehouse_id: _,
        } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            let mut file_path = project_path.clone();
            file_path.push("config.json");

            let mut state_path = project_path.clone();
            state_path.push("objects.json");

            let mut metadata_path = project_path.clone();
            metadata_path.push("metadata/");

            let _schema = deploy_contract::parse_config(file_path.as_path())?;
            let mut state = deploy_contract::parse_state(state_path.as_path())?;

            // if schema.contract.is_none() {
            //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
            // }

            // let mut state = CollectionState::try_read_config(&state_path)?;

            state = mint_nfts::mint_nfts(
                // &schema,
                gas_budget,
                // metadata_path,
                // warehouse_id,
                state,
            )
            .await?;

            state.write(&state_path)?;
        }
        Commands::ParallelMint {
            project_dir,
            gas_budget,
            warehouse_id: _,
        } => {
            let project_path = PathBuf::from(Path::new(project_dir.as_str()));
            let mut file_path = project_path.clone();
            file_path.push("config.json");

            let mut state_path = project_path.clone();
            state_path.push("objects.json");

            let mut metadata_path = project_path.clone();
            metadata_path.push("metadata/");

            // let schema = deploy_contract::parse_config(file_path.as_path())?;
            let state = deploy_contract::parse_state(state_path.as_path())?;

            // if schema.contract.is_none() {
            //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
            // }

            // let mut state = CollectionState::try_read_config(&state_path)?;

            mint_nfts::parallel_mint_nfts(
                // &schema,
                gas_budget,
                // metadata_path,
                // warehouse_id,
                state,
            )
            .await?;

            // io::write_collection_state(&state, &state_path)?;
        }
        Commands::SplitCoin {
            gas_budget,
            amount,
            count,
        } => {
            coin::split(Some(amount), count, gas_budget as u64).await?;
        }
        Commands::CombineCoins { gas_budget } => {
            coin::combine(gas_budget as u64).await?;
        }
        Commands::CheckDependencies { project_dir } => {
            let mut file_path = PathBuf::from(Path::new(project_dir.as_str()));
            file_path.push("contract/Move.toml");

            let toml_string: String =
                fs::read_to_string(file_path.clone())?.parse()?;

            let mut move_toml: MoveToml =
                toml::from_str(toml_string.as_str()).unwrap();

            let url = "https://github.com/Origin-Byte/program-registry";

            let temp_dir =
                TempDir::new().expect("Failed to create temporary directory");

            let repo = match Repository::clone(url, temp_dir.path()) {
                Ok(repo) => repo,
                Err(e) => return Err(anyhow!("failed to clone: {}", e)),
            };

            if !repo.is_empty()? {
                println!("Fetched Program Registry successfully");
            } else {
                return Err(anyhow!(
                    "Something went wrong while accessing the Program Registry"
                ));
            }

            let file_name = PathBuf::from("registry-main.json");
            let mut map_path = temp_dir.path().to_path_buf();
            map_path.push(&file_name);

            let package_map = PackageMap::read(&map_path)?;

            // Note: This code block assumes that there is only one folder
            // in the build folder, which is the case.
            let mut build_info_path =
                PathBuf::from(Path::new(project_dir.as_str()));

            build_info_path.push("contract/build/");
            let mut paths = fs::read_dir(&build_info_path).unwrap();

            if let Some(path) = paths.next() {
                build_info_path = path?.path();
                build_info_path.push("BuildInfo.yaml");
            }

            let mut info = BuildInfo::read_yaml(&build_info_path)?;

            info.packages.make_name_canonical();

            move_toml.update_toml(&package_map);

            let mut toml_string = toml::to_string_pretty(&move_toml)?;

            toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

            let mut file = File::create(file_path)?;
            file.write_all(toml_string.as_bytes())?;
        }
    }

    Ok(())
}
