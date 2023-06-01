pub mod aws;
pub mod cli;
pub mod consts;
pub mod endpoints;
pub mod err;
pub mod io;
pub mod models;
pub mod prelude;

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    models::{
        dependencies::PackageMap,
        toml::{BuildInfo, MoveToml},
    },
    prelude::*,
};
use byte_cli::consts::{BPS_100_PERCENT, CONFIG_FILENAME, PROJECT_FILENAME};
use convert_case::{Case, Casing};
use endpoints::*;

use anyhow::Result;
use clap::Parser;
use console::style;

use gutenberg::{
    models::{
        collection::CollectionData,
        nft::{burn::Burn, NftData},
        settings::{
            royalties::Share, MintPolicies, Orderbook, RequestPolicies,
            RoyaltyPolicy, Settings,
        },
        Address,
    },
    schema::SchemaBuilder,
    Schema,
};
use io::{LocalRead, LocalWrite};
use models::project::Project;
use rust_sdk::coin;
use uploader::writer::Storage;

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

            let keystore = rust_sdk::utils::get_keystore().await?;
            let sender = rust_sdk::utils::get_active_address(&keystore)?;
            let sender_string = Address::new(sender.to_string())?;

            let nft_type = name.as_str().to_case(Case::Pascal);

            let project = Project::new(name.clone(), sender);

            let royalties = Some(RoyaltyPolicy::new(
                BTreeSet::from([Share::new(sender_string, BPS_100_PERCENT)]),
                royalty_bps as u64,
            ));

            let schema = Schema::new(
                CollectionData::new(name, None, None, None, vec![], None),
                NftData::new(nft_type, Burn::Permissionless, false),
                Settings::new(
                    royalties,
                    MintPolicies::new(Some(supply as u64), true, true),
                    RequestPolicies::new(true, false, false),
                    None,
                    Orderbook::Protected,
                ),
            );

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
            let map_path = PathBuf::from(Path::new("versions/versions.json"));

            let toml_string: String = fs::read_to_string(file_path)?.parse()?;

            let move_toml: MoveToml =
                toml::from_str(toml_string.as_str()).unwrap();

            // println!("Move TOML: {:?}", move_toml);

            // println!("************************************");

            let package_map = PackageMap::read(&map_path)?;
            // println!("Protocol MAP: {:?}", package_map);

            println!("************************************");

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

            info.packages.filter_for_originbyte();
            info.packages.make_name_canonical();
            println!("Build Info: {:?}", info);

            println!("************************************");

            let dependencies =
                move_toml.get_contracts_with_fall_back(&package_map, &info);
            println!("Dependencies: {:?}", dependencies);

            println!("************************************");

            let to_update = models::toml::get_dependencies_to_update(
                &dependencies,
                &package_map,
            );

            println!("To UPDATE: {:?}", to_update);
        }
    }

    Ok(())
}
