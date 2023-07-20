pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;

use crate::io::LocalRead;
use crate::models::Accounts;
use anyhow::{anyhow, Result};
use byte_cli::SchemaBuilder;
use clap::Parser;
use cli::{Cli, Commands};
use console::style;
use endpoints::*;
use io::LocalWrite;
use package_manager::toml::{self as move_toml, MoveToml};
use package_manager::{self, get_program_registry, Network};
use rust_sdk::coin;
use rust_sdk::utils::get_context;
use std::env;
use std::io::{Read, Write};
use std::path::Path;
use std::str::FromStr;
use std::{
    fs::{self, File},
    path::PathBuf,
};
use sui_sdk::types::base_types::ObjectID;
use uploader::writer::Storage;

#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {
            println!(
                "\n{}{}",
                consts::KIWI_EMOJI,
                style("Process completed.").green().bold().on_bright()
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
        Commands::ConfigSimple { name, project_dir } => {
            // Input
            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            if Path::new(&project_path).exists() {} // TODO

            let (schema, project) = config_simple::init_schema(&name).await?;

            // Output
            schema.write_json(&schema_path)?;
            project.write_json(&project_path)?;
        }
        Commands::ConfigCollection { name, project_dir } => {
            // Input
            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);
            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            // Logic
            let mut builder = SchemaBuilder::read_json(&schema_path)?;
            let project;

            (builder, project) =
                config_collection::init_collection_config(builder).await?;

            // Output
            builder.write_json(&schema_path)?;
            project.write_json(&project_path)?;
        }
        Commands::ConfigUpload { name, project_dir } => {
            let upload_path =
                io::get_upload_filepath(name.as_str(), &project_dir);

            // Logic
            let uploader = config_upload::init_upload_config()?;

            // Output
            uploader.write_json(&upload_path)?;
        }
        Commands::UploadImages { name, project_dir } => {
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
        // Commands::GenerateContract { name, project_dir } => {
        //     // Input
        //     let schema_path =
        //         io::get_schema_filepath(name.as_str(), &project_dir);
        //     let contract_dir =
        //         io::get_contract_path(name.as_str(), &project_dir);

        //     // Logic
        //     let schema = deploy_contract::parse_config(schema_path.as_path())?;
        //     deploy_contract::generate_contract(schema, contract_dir.as_path())?;
        // }
        Commands::DeployContract {
            name,
            network,
            project_dir,
            gas_budget,
        } => {
            // Input
            let project_path =
                io::get_project_filepath(name.as_str(), &project_dir);

            // TODO
            let _contract_dir =
                io::get_contract_path(name.as_str(), &project_dir);

            let schema_path =
                io::get_schema_filepath(name.as_str(), &project_dir);

            let byte_path = io::get_byte_path(&None);
            let accounts = Accounts::read_json(&byte_path)?;

            // TODO
            let _main_account = accounts.get_main_account();

            let network = if network.is_some() {
                Network::from_str(network.unwrap().as_str())
                    .map_err(|err| anyhow!("Invalid network: {:?}", err))?
            } else {
                Network::Mainnet
            };

            // Logic
            // TODO
            let _theme = cli::get_dialoguer_theme();

            let mut state =
                deploy_contract::parse_state(project_path.as_path())?;

            let schema = deploy_contract::parse_config(schema_path.as_path())?;
            let accounts = Accounts::read_json(&byte_path)?;

            deploy_contract::prepare_publish_contract(
                &mut state, &accounts, name, &schema, gas_budget, network,
            )
            .await?;

            // let response = deploy_contract::publish_contract(
            //     gas_budget,
            //     &PathBuf::from(contract_dir.as_path()),
            // )
            // .await?;

            // deploy_contract::process_effects(&mut state, response).await?;

            // Output
            // state.write_json(&project_path)?;
        }
        // Commands::CreateWarehouse {
        //     name,
        //     project_dir,
        //     gas_budget,
        // } => {
        //     // Input
        //     let schema_path =
        //         io::get_schema_filepath(name.as_str(), &project_dir);

        //     let project_path =
        //         io::get_project_filepath(name.as_str(), &project_dir);

        //     // Input
        //     let toml_path = io::get_toml_path(name.as_str(), &project_dir);

        //     // Logic
        //     let toml_string: String =
        //         fs::read_to_string(toml_path.clone())?.parse()?;

        //     let move_toml: MoveToml =
        //         toml::from_str(toml_string.as_str()).unwrap();

        //     // Logic
        //     let schema = deploy_contract::parse_config(schema_path.as_path())?;
        //     let mut state =
        //         deploy_contract::parse_state(project_path.as_path())?;

        //     if state.package_id.is_none() {
        //         return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
        //     }

        //     create_warehouse::create_warehouse(
        //         &schema, gas_budget, &move_toml, &mut state,
        //     )
        //     .await?;

        //     // Output
        //     state.write_json(&project_path)?;
        // }
        // Commands::MintNfts {
        //     name,
        //     project_dir,
        //     gas_budget,
        //     warehouse_id,
        //     mint_cap_id,
        // } => {
        //     // Input
        //     let schema_path =
        //         io::get_schema_filepath(name.as_str(), &project_dir);

        //     let project_path =
        //         io::get_project_filepath(name.as_str(), &project_dir);

        //     let (_, post_upload) =
        //         io::get_upload_metadata(name.as_str(), &project_dir);

        //     // Logic
        //     // TODO: Replace this logic with the our IO Trait
        //     let schema = deploy_contract::parse_config(schema_path.as_path())?;
        //     let mut state =
        //         deploy_contract::parse_state(project_path.as_path())?;

        //     if state.package_id.is_none() {
        //         return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
        //     }

        //     state = mint_nfts::mint_nfts(
        //         &schema,
        //         gas_budget,
        //         warehouse_id,
        //         mint_cap_id,
        //         post_upload,
        //         state,
        //     )
        //     .await?;

        //     // Output
        //     state.write_json(&project_path)?;
        // }
        Commands::ListCoins {} => {
            let wallet_ctx = get_context().await.unwrap();
            let client = wallet_ctx.get_client().await?;
            let sender = wallet_ctx.config.active_address.unwrap();

            let coin_list = coin::list_coins(&client, sender).await?;

            println!("{}", coin_list);
        }
        Commands::SplitCoin {
            coin_id,
            gas_budget,
            amount,
            count,
            gas_id,
        } => {
            let wallet_ctx = get_context().await.unwrap();
            let client = wallet_ctx.get_client().await?;
            let sender = wallet_ctx.config.active_address.unwrap();

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

            let coin_list = coin::list_coins(&client, sender).await?;
            println!("{}", coin_list);
        }
        Commands::CombineCoins { gas_budget, gas_id } => {
            let wallet_ctx = get_context().await.unwrap();
            let client = wallet_ctx.get_client().await?;
            let sender = wallet_ctx.config.active_address.unwrap();

            let gas_id =
                ObjectID::from_str(gas_id.as_str()).map_err(|err| {
                    anyhow!(r#"Unable to parse gas-id object: {err}"#)
                })?;

            coin::combine(gas_budget as u64, gas_id).await?;

            let coin_list = coin::list_coins(&client, sender).await?;
            println!("{}", coin_list);
        }
        Commands::CheckDependencies {
            name,
            network,
            project_dir,
        } => {
            // Input
            let toml_path = io::get_toml_path(name.as_str(), &project_dir);

            let network = if network.is_some() {
                Network::from_str(network.unwrap().as_str())
                    .map_err(|err| anyhow!("Invalid network: {:?}", err))?
            } else {
                Network::Mainnet
            };

            let registry = get_program_registry(&network)?;

            // Logic
            let toml_string: String =
                fs::read_to_string(toml_path.clone())?.parse()?;

            let mut move_toml: MoveToml =
                toml::from_str(toml_string.as_str()).unwrap();

            move_toml.update_toml(&registry);

            let mut toml_string = toml::to_string_pretty(&move_toml)?;

            toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

            // Output
            let mut file = File::create(toml_path)?;
            file.write_all(toml_string.as_bytes())?;
        }
        Commands::UseEnv {
            network,
            name,
            project_dir,
        } => {
            let mut project_dir = match project_dir {
                Some(pj_dir) => PathBuf::from(Path::new(pj_dir.as_str())),
                None => match &name {
                    // If there is a project name but no project dir, then
                    // default to `.byte` directory
                    Some(_) => {
                        dirs::home_dir().unwrap().join(".byte/projects/")
                    }
                    None => env::current_dir()?,
                },
            };

            if let Some(name) = name {
                project_dir.push(format!("{}/contract", name));
            }

            let network = Network::from_str(network.as_str())
                .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

            let flavours_path = match network {
                Network::Mainnet => project_dir.join("flavours/Move-main.toml"),
                Network::Testnet => project_dir.join("flavours/Move-test.toml"),
            };

            println!("Project dir: {:?}", project_dir);
            println!("flavours_path: {:?}", flavours_path);

            // Logic
            // Open the source file for reading
            let mut source_file = fs::File::open(flavours_path)?;

            // Create or open the destination file for writing
            let mut destination_file =
                fs::File::create(project_dir.join("Move.toml"))?;

            // Read the contents of the source file
            let mut buffer = Vec::new();
            source_file.read_to_end(&mut buffer)?;

            // Output
            // Write the contents to the destination file
            destination_file.write_all(&buffer)?;
        }
        Commands::AddProfile { root_dir } => {
            let byte_path = io::get_byte_path(&root_dir);
            let mut accounts = Accounts::read_json(&byte_path)?;

            let result = add_profile::add_profile(&mut accounts).await?;

            accounts.write_json(&byte_path.as_path())?;

            let body = result.text().await?;
            println!("{:?}", body);
        }
        Commands::Signup { root_dir } => {
            let byte_path = io::get_byte_path(&root_dir);
            let mut accounts = Accounts::read_json(&byte_path)?;

            let result = signup::signup(&mut accounts).await?;

            accounts.write_json(&byte_path.as_path())?;

            let body = result.text().await?;
            println!("Response: {:?}", body);
        }
        Commands::SwitchAccount { email, root_dir } => {
            let byte_path = io::get_byte_path(&root_dir);
            let mut accounts = Accounts::read_json(&byte_path)?;

            if !accounts.check_if_registered(&email) {
                return Err(anyhow!(
                    "Email account not present in local storage"
                ));
            }

            accounts.set_main(email);
            accounts.write_json(&byte_path.as_path())?;
        } // TOOD: Add back feature
          // Commands::ParallelMint {
          //     name,
          //     project_dir,
          //     gas_budget,
          //     main_gas_id,
          //     minor_gas_id,
          // } => {
          //     // Input
          //     let _schema_path =
          //         io::get_schema_filepath(name.as_str(), &project_dir);

          //     let project_path =
          //         io::get_project_filepath(name.as_str(), &project_dir);

          //     // Logic
          //     // let schema = deploy_contract::parse_config(file_path.as_path())?;
          //     let state = deploy_contract::parse_state(project_path.as_path())?;

          //     // if schema.contract.is_none() {
          //     //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
          //     // }

          //     // let mut state = CollectionState::try_read_config(&state_path)?;

          //     let main_gas_id = ObjectID::from_str(main_gas_id.as_str())
          //         .map_err(|err| {
          //             anyhow!(r#"Unable to parse main-gas-id object: {err}"#)
          //         })?;
          //     let minor_gas_id = ObjectID::from_str(minor_gas_id.as_str())
          //         .map_err(|err| {
          //             anyhow!(r#"Unable to parse minor-gas-id object: {err}"#)
          //         })?;

          //     mint_nfts::parallel_mint_nfts(
          //         name,
          //         gas_budget,
          //         state,
          //         main_gas_id,
          //         minor_gas_id,
          //     )
          //     .await?;

          //     // Output
          //     // io::write_collection_state(&state, &state_path)?;
          // }
    }

    Ok(())
}
