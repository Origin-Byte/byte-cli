use anyhow::{anyhow, Result};
use console::style;
use gutenberg::Schema;
use package_manager::{
    move_lib::PackageMap,
    toml::{self as move_toml, MoveToml},
    version::Version,
};
use rust_sdk::coin;
use std::io::Write;
use std::path::Path;

use rust_sdk::{collection_state::CollectionState, publish};
use std::fs::{self, File};

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

pub fn parse_state(config_file: &Path) -> Result<CollectionState> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find state file "{}": {err}
Call `byte_cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, CollectionState>(file)
        .map_err(|err| anyhow!(r#"ERR TODO: {err}."#))
}

pub fn generate_contract(
    schema: &Schema,
    contract_dir: &Path,
    package_map: &PackageMap,
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
    let package_path = &contract_dir.join("Move.toml");
    let mut package_file = File::create(package_path).map_err(|err| {
        anyhow!(
            r#"Could not create Move.toml "{}": {err}"#,
            package_path.display()
        )
    })?;

    let module_name = schema.collection().name();

    let move_toml = MoveToml::get_toml(
        module_name.as_str(),
        package_map,
        &vec![
            String::from("Sui"),
            String::from("Originmate"),
            String::from("NftProtocol"),
            String::from("Launchpad"),
            String::from("LiquidityLayerV1"),
        ],
        &Version::from_string("0.0.1")?,
    )?;

    let mut toml_string = toml::to_string_pretty(&move_toml)?;

    toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

    // Output
    package_file.write_all(toml_string.as_bytes())?;

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
) -> Result<CollectionState> {
    let wallet_ctx = rust_sdk::utils::get_context().await?;

    let gas_coin =
        rust_sdk::utils::get_coin_ref(&coin::get_max_coin(&wallet_ctx).await?);

    let collection_state = publish::publish_contract_and_pay(
        &wallet_ctx,
        contract_dir,
        gas_coin,
        gas_budget as u64,
    )
    .await?;

    Ok(collection_state)
}
