pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;

use crate::io::LocalRead;
use anyhow::{anyhow, Result};
use byte_cli::SchemaBuilder;
use clap::Parser;
use cli::{
    Cli, ClientCommands, CoinCommands, CollectionCommands, ImageCommands,
    MoveCommands,
};
use endpoints::collection::codegen;
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

/// Main entry point for the application.
/// This is an asynchronous function due to network and IO operations.
#[tokio::main]
async fn main() {
    match run().await {
        Ok(()) => {}
        Err(err) => {
            println!("\n{}", err,);
            std::process::exit(1);
        }
    }
}

/// Core runtime logic of the application.
///
/// Parses command-line arguments and executes the corresponding command.
/// This function is asynchronous due to potential network and IO operations.
async fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Collection { cmd } => {
            match cmd {
                CollectionCommands::ConfigBasic { name, project_dir } => {
                    // Input
                    let project_path =
                        io::get_project_filepath(name.as_str(), &project_dir);

                    let project_test_path = io::get_project_test_filepath(
                        name.as_str(),
                        &project_dir,
                    );

                    let schema_path =
                        io::get_schema_filepath(name.as_str(), &project_dir);

                    if Path::new(&project_path).exists() {} // TODO

                    let (schema, project) =
                        collection::config_basic::init_schema(&name).await?;

                    // Output
                    schema.write_json(&schema_path)?;
                    project.write_json(&project_path)?;
                    project.write_json(&project_test_path)?;
                }
                CollectionCommands::Config { name, project_dir } => {
                    // Input
                    let project_path =
                        io::get_project_filepath(name.as_str(), &project_dir);
                    let schema_path =
                        io::get_schema_filepath(name.as_str(), &project_dir);

                    // Logic
                    let mut builder = SchemaBuilder::read_json(&schema_path)?;
                    let project;

                    (builder, project) =
                        collection::config::init_collection_config(builder)
                            .await?;

                    // Output
                    builder.write_json(&schema_path)?;
                    project.write_json(&project_path)?;
                }
                CollectionCommands::Codegen { name, project_dir } => {
                    // Input
                    let schema_path =
                        io::get_schema_filepath(name.as_str(), &project_dir);
                    let contract_dir =
                        io::get_contract_path(name.as_str(), &project_dir);

                    // Logic
                    let schema = codegen::parse_config(schema_path.as_path())?;
                    codegen::gen_contract(contract_dir.as_path(), &schema)
                        .await?;
                }
            }
        }
        Cli::Images { cmd } => {
            match cmd {
                ImageCommands::Config { name, project_dir } => {
                    // IO Read
                    let upload_path =
                        io::get_upload_filepath(name.as_str(), &project_dir);

                    // Logic
                    let uploader = images::config::init_upload_config()?;

                    // IO Write
                    uploader.write_json(&upload_path)?;
                }
                ImageCommands::Upload { name, project_dir } => {
                    // IO Read
                    let assets_path =
                        io::get_assets_path(name.as_str(), &project_dir);
                    let (pre_upload, post_upload) =
                        io::get_upload_metadata(name.as_str(), &project_dir);
                    let upload_config_path =
                        io::get_upload_filepath(name.as_str(), &project_dir);

                    // Logic
                    let uploader = Storage::read_json(&upload_config_path)?;

                    images::upload::deploy_assets(
                        &uploader,
                        assets_path,
                        pre_upload,
                        post_upload,
                    )
                    .await?
                }
            }
        }
        Cli::Client { cmd } => match cmd {
            ClientCommands::PublishCollection {
                name,
                network,
                project_dir,
                gas_coin,
                gas_budget,
            } => {
                // Input
                let network = Network::from_str(network.as_str())
                    .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                let project_path = io::get_project_for_network(
                    name.as_str(),
                    &project_dir,
                    &network,
                );

                let contract_dir =
                    io::get_contract_path(name.as_str(), &project_dir);

                // Logic
                let mut state = client::deploy_contract::parse_state(
                    project_path.as_path(),
                )?;

                client::deploy_contract::publish_contract(
                    &mut state,
                    gas_coin,
                    gas_budget,
                    network,
                    &contract_dir,
                )
                .await?;

                // IO Write
                state.write_json(&project_path)?;
            }
            ClientCommands::CreateWarehouse {
                name,
                network,
                project_dir,
                gas_coin,
                gas_budget,
            } => {
                // Input
                let network = Network::from_str(network.as_str())
                    .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                let project_path = io::get_project_for_network(
                    name.as_str(),
                    &project_dir,
                    &network,
                );

                let schema_path =
                    io::get_schema_filepath(name.as_str(), &project_dir);

                // Logic
                let schema = codegen::parse_config(schema_path.as_path())?;
                let mut state = client::deploy_contract::parse_state(
                    project_path.as_path(),
                )?;

                if state.package_id.is_none() {
                    return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
                }

                client::create_warehouse::create_warehouse(
                    &schema, gas_budget, gas_coin, &mut state, &network,
                )
                .await?;

                // Output
                state.write_json(&project_path)?;
            }
            ClientCommands::MintNfts {
                name,
                network,
                amount,
                batches,
                project_dir,
                gas_budget,
                warehouse_id,
                mint_cap_id,
            } => {
                // Input
                let schema_path =
                    io::get_schema_filepath(name.as_str(), &project_dir);

                let (_, post_upload) =
                    io::get_upload_metadata(name.as_str(), &project_dir);

                let network = Network::from_str(network.as_str())
                    .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                let project_path = io::get_project_for_network(
                    name.as_str(),
                    &project_dir,
                    &network,
                );

                // Logic
                // TODO: Replace this logic with our IO Trait
                let schema = codegen::parse_config(schema_path.as_path())?;
                let mut state = client::deploy_contract::parse_state(
                    project_path.as_path(),
                )?;

                if state.package_id.is_none() {
                    return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
                }

                state = client::mint_nfts::mint_nfts(
                    &schema,
                    gas_budget,
                    warehouse_id,
                    mint_cap_id,
                    post_upload,
                    state,
                    amount,
                    batches,
                    &network,
                )
                .await?;

                // Output
                state.write_json(&project_path)?;
            } /* TOOD: Add back feature
               * Commands::ParallelMint {
               *     name,
               *     project_dir,
               *     gas_budget,
               *     main_gas_id,
               *     minor_gas_id,
               * } => {
               *     // Input
               *     let _schema_path =
               *         io::get_schema_filepath(name.as_str(),
               * &project_dir); */

              /*     let project_path =
               *         io::get_project_filepath(name.as_str(),
               * &project_dir); */

              /*     // Logic
               *     // let schema = deploy_contract::parse_config(file_path.as_path())?;
               *     let state =
               * deploy_contract::parse_state(project_path.as_path())?; */

              /*     // if schema.contract.is_none() {
               *     //     return Err(anyhow!("Error: Could not find contract ID in config file. Make sure you run the command `deploy-contract`"));
               *     // } */

              /*     // let mut state = CollectionState::try_read_config(&state_path)?; */

              /*     let main_gas_id =
               * ObjectID::from_str(main_gas_id.as_str())
               *         .map_err(|err| {
               *             anyhow!(r#"Unable to parse main-gas-id object:
               * {err}"#)         })?;
               *     let minor_gas_id =
               * ObjectID::from_str(minor_gas_id.as_str())
               *         .map_err(|err| {
               *             anyhow!(r#"Unable to parse minor-gas-id object:
               * {err}"#)         })?; */

              /*     mint_nfts::parallel_mint_nfts(
               *         name,
               *         gas_budget,
               *         state,
               *         main_gas_id,
               *         minor_gas_id,
               *     )
               *     .await?; */

              /*     // Output
               *     // io::write_collection_state(&state, &state_path)?;
               * } */
        },
        Cli::Coin { cmd } => match cmd {
            CoinCommands::List {} => {
                let wallet_ctx = get_context().await.unwrap();
                let client = wallet_ctx.get_client().await?;
                let sender = wallet_ctx.config.active_address.unwrap();

                let coin_list = coin::list_coins(&client, sender).await?;

                println!("{}", coin_list);
            }
            CoinCommands::Split {
                coin_id,
                gas_budget,
                amount,
                count,
                gas_coin,
            } => {
                let wallet_ctx = get_context().await.unwrap();
                let client = wallet_ctx.get_client().await?;
                let sender = wallet_ctx.config.active_address.unwrap();

                let gas_id = match gas_coin {
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
            CoinCommands::Melt {
                gas_budget,
                gas_coin,
            } => {
                let wallet_ctx = get_context().await.unwrap();
                let client = wallet_ctx.get_client().await?;
                let sender = wallet_ctx.config.active_address.unwrap();

                let gas_id =
                    ObjectID::from_str(gas_coin.as_str()).map_err(|err| {
                        anyhow!(r#"Unable to parse gas-id object: {err}"#)
                    })?;

                coin::combine(gas_budget as u64, gas_id).await?;

                let coin_list = coin::list_coins(&client, sender).await?;
                println!("{}", coin_list);
            }
        },
        Cli::Move { cmd } => {
            match cmd {
                MoveCommands::UpdateDependencies {
                    name,
                    network,
                    project_dir,
                } => {
                    // Input
                    let toml_path =
                        io::get_toml_path(name.as_str(), &project_dir);

                    let network_ = network.as_str();

                    let network = Network::from_str(network_)
                        .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                    let registry = get_program_registry(&network)?;

                    // Logic
                    let toml_string: String =
                        fs::read_to_string(toml_path.clone())?.parse()?;

                    let mut move_toml: MoveToml =
                        toml::from_str(toml_string.as_str()).unwrap();

                    if let Some(flavor) = move_toml.package.flavor {
                        if network_ != flavor.to_str() {
                            return Err(anyhow!(
                                "Network '{}' does not match flavor '{}'",
                                network,
                                flavor.to_str()
                            ));
                        }
                    }

                    move_toml.update_toml(&registry);

                    let mut toml_string = toml::to_string_pretty(&move_toml)?;

                    toml_string =
                        move_toml::add_vertical_spacing(toml_string.as_str());

                    // Output
                    let mut file = File::create(toml_path)?;
                    file.write_all(toml_string.as_bytes())?;
                }
                MoveCommands::CheckDependencies {
                    name,
                    network,
                    project_dir,
                } => {
                    // Input
                    let toml_path =
                        io::get_toml_path(name.as_str(), &project_dir);

                    let network_ = network.as_str();

                    let network = Network::from_str(network_)
                        .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                    let registry = get_program_registry(&network)?;

                    // Logic
                    let toml_string: String =
                        fs::read_to_string(toml_path.clone())?.parse()?;

                    let move_toml: MoveToml =
                        toml::from_str(toml_string.as_str()).unwrap();

                    if let Some(flavor) = move_toml.package.flavor {
                        if network_ != flavor.to_str() {
                            return Err(anyhow!(
                                "Network '{}' does not match flavor '{}'",
                                network,
                                flavor.to_str()
                            ));
                        }
                    }

                    move_toml.check_updates(&registry);
                }
                MoveCommands::LoadEnv {
                    name,
                    network,
                    project_dir,
                } => {
                    let mut project_dir = match project_dir {
                        Some(pj_dir) => {
                            PathBuf::from(Path::new(pj_dir.as_str()))
                        }
                        None => match &name {
                            // If there is a project name but no project dir,
                            // then default to
                            // `.byte` directory
                            Some(_) => dirs::home_dir()
                                .unwrap()
                                .join(".byte/projects/"),
                            None => env::current_dir()?,
                        },
                    };

                    if let Some(name) = name {
                        project_dir.push(format!("{}/contract", name));
                    }

                    let network = Network::from_str(network.as_str())
                        .map_err(|err| anyhow!("Invalid network: {:?}", err))?;

                    let flavours_path = match network {
                        Network::Mainnet => {
                            project_dir.join("flavours/Move-main.toml")
                        }
                        Network::Testnet => {
                            project_dir.join("flavours/Move-test.toml")
                        }
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
            }
        }
    }

    Ok(())
}
