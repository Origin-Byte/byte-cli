use anyhow::{anyhow, Result};
use console::style;
use gutenberg::Schema;
use package_manager::{
    move_lib::PackageMap,
    toml::{self as move_toml, MoveToml},
    version::Version,
};
use rust_sdk::{coin, consts::VOLCANO_EMOJI, utils::get_context};
use std::io::Write;
use std::path::Path;
use sui_sdk::rpc_types::{
    SuiTransactionBlockEffects, SuiTransactionBlockResponse,
};
use terminal_link::Link;

use std::sync::mpsc::channel;
use tokio::task::JoinSet;

use rust_sdk::{collection_state::ObjectType as OBObjectType, publish};
use std::fs::{self, File};

use crate::models::project::{
    AdminObjects, CollectionObjects, MintCap, Project,
};

pub fn parse_config(config_file: &Path) -> Result<Schema> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find configuration file "{}": {err}
Call `byte_cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Schema>(file).map_err(|err|anyhow!(r#"Could not parse configuration file "{}": {err}
Call `byte_cli init-collection-config to initialize the configuration file again."#, config_file.display()))
}

pub fn parse_state(config_file: &Path) -> Result<Project> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find state file "{}": {err}
Call `byte_cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Project>(file)
        .map_err(|err| anyhow!(r#"ERR TODO: {err}."#))
}

pub fn generate_contract(
    schema: &Schema,
    contract_dir: &Path,
    main_registry: &PackageMap,
    test_registry: &PackageMap,
    version: Option<String>,
) -> Result<()> {
    println!("{} Generating contract", style("WIP").cyan().bold());

    let sources_dir = &contract_dir.join("sources");
    let _ = fs::remove_dir_all(sources_dir);
    fs::create_dir_all(sources_dir).map_err(|err| {
        anyhow!(
            r#"Could not create directory "{}": {err}"#,
            sources_dir.display()
        )
    })?;

    // Write Move.toml
    // Create the directory if it doesn't exist
    fs::create_dir_all(&contract_dir.join("flavours/"))?;

    let main_toml_path = &contract_dir.join("flavours/Move-main.toml");
    let mut mail_toml_file = File::create(main_toml_path).map_err(|err| {
        anyhow!(
            r#"Could not create Move.toml "{}": {err}"#,
            main_toml_path.display()
        )
    })?;

    // Write Move-test.toml
    let test_toml_path = &contract_dir.join("flavours/Move-test.toml");
    let mut test_toml_file = File::create(test_toml_path).map_err(|err| {
        anyhow!(
            r#"Could not create Move-test.toml "{}": {err}"#,
            test_toml_path.display()
        )
    })?;

    let module_name = schema.package_name();

    let main_toml_string =
        write_toml_string(module_name.as_str(), &version, &main_registry)?;

    let test_toml_string =
        write_toml_string(module_name.as_str(), &version, &test_registry)?;

    // Output
    mail_toml_file.write_all(main_toml_string.as_bytes())?;
    test_toml_file.write_all(test_toml_string.as_bytes())?;

    // Copy Main Move.toml
    fs::copy(main_toml_path, &contract_dir.join("Move.toml"))?;

    // Write Move contract
    let move_path = &sources_dir.join(format!("{module_name}.move"));
    let mut move_file = File::create(move_path).map_err(|err| {
        anyhow!(r#"Could not create "{}": {err}"#, move_path.display())
    })?;

    schema.write_move(&mut move_file).map_err(|err| {
        anyhow!(
            r#"Could not Move contract "{}": {err}"#,
            move_path.display()
        )
    })?;

    println!("{} Generating contract", style("DONE").green().bold());

    Ok(())
}

pub async fn publish_contract(
    gas_budget: usize,
    contract_dir: &Path,
) -> Result<SuiTransactionBlockResponse> {
    let wallet_ctx = rust_sdk::utils::get_context().await?;

    let gas_coin =
        rust_sdk::utils::get_coin_ref(&coin::get_max_coin(&wallet_ctx).await?);

    let response = publish::publish_contract_and_pay(
        &wallet_ctx,
        contract_dir,
        gas_coin,
        gas_budget as u64,
    )
    .await?;

    Ok(response)
}

pub fn write_toml_string(
    module_name: &str,
    version: &Option<String>,
    registry: &PackageMap,
) -> Result<String> {
    let mut move_toml = match version {
        Some(version) => MoveToml::get_toml(
            module_name,
            registry,
            &vec![
                String::from("NftProtocol"),
                String::from("Launchpad"),
                String::from("LiquidityLayerV1"),
            ],
            &vec![String::from("Sui"), String::from("Originmate")],
            &Version::from_string(version.as_str())?,
        )?,
        None => MoveToml::get_toml_latest(
            module_name,
            registry,
            &vec![
                String::from("NftProtocol"),
                String::from("Launchpad"),
                String::from("LiquidityLayerV1"),
            ],
            &vec![String::from("Sui"), String::from("Originmate")],
        )?,
    };

    move_toml.sanitize_output();

    let mut toml_string = toml::to_string_pretty(&move_toml)?;
    toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

    Ok(toml_string)
}

pub async fn process_effects(
    state: &mut Project,
    response: SuiTransactionBlockResponse,
) -> Result<()> {
    let context = get_context().await.unwrap();
    let client = context.get_client().await?;

    println!(
        "{} {}",
        VOLCANO_EMOJI,
        style("Contract has been successfuly deployed on-chain.")
            .green()
            .bold()
    );
    let mut set = JoinSet::new();

    // Creating a channel to send message with package ID
    let (sender, receiver) = channel();

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    let objects_created = effects.created;

    objects_created
        .iter()
        .map(|object| {
            // TODO: Remove this clone
            let object_ = object.clone();
            let client_ = client.clone();
            let sender_ = sender.clone();
            set.spawn(async move {
                publish::print_object(&client_, &object_, sender_).await;
            });
        })
        .for_each(drop);

    let mut i = 1;
    while let Some(res) = set.join_next().await {
        res.unwrap();
        i += 1;
    }

    println!("A total of {} object have been created.", i);

    let mut j = 0;

    // It's three as we are interest in the MintCap, Collection and Package
    // We need to make sure we agree on the number of objects that are recorded
    while j < 3 {
        let object_type = receiver.recv().unwrap();
        match object_type {
            OBObjectType::Package(object_id) => {
                state.package_id = Some(object_id);
                j += 1;
            }
            OBObjectType::MintCap(object_id) => {
                let admin_objs =
                    state.admin_objects.get_or_insert(AdminObjects::empty());
                admin_objs.mint_caps.push(MintCap::new(object_id));
                j += 1;
            }
            OBObjectType::Collection(object_id) => {
                let col_objs = state
                    .collection_objects
                    .get_or_insert(CollectionObjects::empty());

                col_objs.collection = Some(object_id);

                j += 1;
            }
            _ => {}
        }
    }

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network=testnet",
        state.package_id.as_ref().unwrap()
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(())
}
