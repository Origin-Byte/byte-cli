pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;
pub mod prelude;

use anyhow::{anyhow, Result};
use clap::Parser;
use console::style;
use dialoguer::Confirm;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::str::FromStr;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use sui_sdk::types::base_types::ObjectID;

use crate::prelude::*;
use byte_cli::io::{LocalRead, LocalWrite};
use byte_cli::SchemaBuilder;
use endpoints::*;
use package_manager::toml::{self as move_toml, MoveToml};
use rust_sdk::{coin, consts::PRICE_PUBLISH};
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
            // Input
            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            // Logic
            let (schema, project) =
                new_simple::init_schema(&name, supply, royalty_bps).await?;

            // Output
            schema.write_json(&schema_path)?;
            project.write_json(&project_path)?;
        }
        Commands::ConfigCollection {
            name,
            project_dir,
            complete,
        } => {
            // Input
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            // Logic
            let mut builder = SchemaBuilder::read_json(&schema_path)?;

            builder =
                config_collection::init_collection_config(builder, complete)?;

            // Output
            builder.write_json(&schema_path)?;
        }
        Commands::ConfigUpload { name, project_dir } => {
            // Input
            // TODO: These config file is currently not setup
            let upload_path =
                io::get_upload_filepath(name.as_str(), &project_dir);

            // Logic
            let uploader = config_upload::init_upload_config()?;

            // Output
            uploader.write_json(&upload_path)?;
        }
        Commands::Config { name, project_dir } => {
            // Input
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            let upload_path =
                io::get_upload_filepath(name.as_str(), &project_dir);

            // Logic
            let mut builder = SchemaBuilder::read_json(&schema_path)?;

            builder =
                config_collection::init_collection_config(builder, false)?;
            let uploader = config_upload::init_upload_config()?;

            // Output
            builder.write_json(&schema_path)?;
            uploader.write_json(&upload_path)?;
        }
        Commands::DeployAssets { name, project_dir } => {
            // Input
            let assets_path = io::get_assets_path(name.as_str(), &project_dir);
            let (pre_upload, post_upload) =
                io::get_upload_metadata(name.as_str(), &project_dir);

            let upload_config_path =
                io::get_upload_filepath(name.as_str(), &project_dir);

            // Logic
            let uploader = Storage::read_json(&upload_config_path)?;

            deploy_assets::deploy_assets(
                &uploader,
                assets_path,
                pre_upload,
                post_upload,
            )
            .await?
        }
        Commands::GenerateContract {
            name,
            project_dir,
            version,
        } => {
            // Input
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);
            let contract_dir =
                io::get_contract_path(name.as_str(), &project_dir);

            let package_map = io::get_program_registry()?;

            // Logic
            let schema = deploy_contract::parse_config(schema_path.as_path())?;

            deploy_contract::generate_contract(
                &schema,
                contract_dir.as_path(),
                &package_map,
                version,
            )?;
        }
        Commands::DeployContract {
            name,
            project_dir,
            gas_budget,
            skip_generation,
            version,
        } => {
            // Input
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            let contract_dir =
                io::get_contract_path(name.as_str(), &project_dir);

            // Logic
            let schema = deploy_contract::parse_config(schema_path.as_path())?;

            if !skip_generation {
                let package_map = io::get_program_registry()?;

                deploy_contract::generate_contract(
                    &schema,
                    contract_dir.as_path(),
                    &package_map,
                    version,
                )?;
            }

            let theme = get_dialoguer_theme();

            let agreed = Confirm::with_theme(&theme)
                .with_prompt(format!(
                "This action has a cost of {} MIST. Do you want to proceed?",
                PRICE_PUBLISH,
                ))
                .interact()
                .unwrap();

            if agreed {
                let state = deploy_contract::publish_contract(
                    gas_budget,
                    &PathBuf::from(contract_dir.as_path()),
                )
                .await?;

                // Output

                // TODO: This project.json will not deserialize to this struct
                state.write_json(&project_path)?;
            }
        }
        Commands::MintNfts {
            name,
            project_dir,
            gas_budget,
            warehouse_id: _,
        } => {
            // Input
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            let _metadata_path =
                io::get_assets_path(name.as_str(), &project_dir);

            // Logic
            // TODO: Replace this logic with the our IO Trait
            let _schema = deploy_contract::parse_config(schema_path.as_path())?;
            let mut state =
                deploy_contract::parse_state(project_path.as_path())?;

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

            // Output
            // TODO: This project.json will not deserialize to this struct
            state.write_json(&project_path)?;
        }
        Commands::ParallelMint {
            name,
            project_dir,
            gas_budget,
            main_gas_id,
            minor_gas_id,
        } => {
            // Input
            let _schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            let _metadata_path =
                io::get_assets_path(name.as_str(), &project_dir);

            // Logic
            // let schema = deploy_contract::parse_config(file_path.as_path())?;
            let state = deploy_contract::parse_state(project_path.as_path())?;

            // if schema.contract.is_none() {
            //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
            // }

            // let mut state = CollectionState::try_read_config(&state_path)?;

            let main_gas_id = ObjectID::from_str(main_gas_id.as_str())
                .map_err(|err| {
                    anyhow!(r#"Unable to parse main-gas-id object: {err}"#)
                })?;
            let minor_gas_id = ObjectID::from_str(minor_gas_id.as_str())
                .map_err(|err| {
                    anyhow!(r#"Unable to parse minor-gas-id object: {err}"#)
                })?;

            mint_nfts::parallel_mint_nfts(
                name,
                gas_budget,
                state,
                main_gas_id,
                minor_gas_id,
            )
            .await?;

            // Output
            // io::write_collection_state(&state, &state_path)?;
        }
        Commands::ListCoins {} => {
            let coin_list = coin::list_coins().await?;

            println!("{}", coin_list);
        }
        Commands::SplitCoin {
            coin_id,
            gas_budget,
            amount,
            count,
            gas_id,
        } => {
            let gas_id = match gas_id {
                Some(gas_id) => Some(
                    ObjectID::from_str(gas_id.as_str()).map_err(|err| {
                        anyhow!(r#"Unable to parse gas-id object: {err}"#)
                    })?,
                ),
                None => None,
            };

            let coin_id =
                ObjectID::from_str(coin_id.as_str()).map_err(|err| {
                    anyhow!(r#"Unable to parse coin-id object: {err}"#)
                })?;

            coin::split(coin_id, amount, count, gas_budget as u64, gas_id)
                .await?;

            let coin_list = coin::list_coins().await?;
            println!("{}", coin_list);
        }
        Commands::CombineCoins { gas_budget, gas_id } => {
            let gas_id =
                ObjectID::from_str(gas_id.as_str()).map_err(|err| {
                    anyhow!(r#"Unable to parse gas-id object: {err}"#)
                })?;

            coin::combine(gas_budget as u64, gas_id).await?;

            let coin_list = coin::list_coins().await?;
            println!("{}", coin_list);
        }
        Commands::CheckDependencies { name, project_dir } => {
            // Input
            let toml_path = io::get_toml_path(name.as_str(), &project_dir);
            let package_map = io::get_program_registry()?;

            // Logic
            let toml_string: String =
                fs::read_to_string(toml_path.clone())?.parse()?;

            let mut move_toml: MoveToml =
                toml::from_str(toml_string.as_str()).unwrap();

            move_toml.update_toml(&package_map);

            let mut toml_string = toml::to_string_pretty(&move_toml)?;

            toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

            // Output
            let mut file = File::create(toml_path)?;
            file.write_all(toml_string.as_bytes())?;
        }
    }

    Ok(())
}
