use anyhow::{anyhow, Result};
use byte_cli::consts::{
    ORIGINMATE_PACKAGE_COMMIT_TEST, PROTOCOL_PACKAGE_COMMIT_TEST,
    SUI_PACKAGE_COMMIT_TEST,
};
use console::style;
use gutenberg::{package, Schema};
use rust_sdk::coin;
use serde::Serialize;
use std::io::Write;
use std::path::{Path, PathBuf};

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

pub fn generate_contract(schema: &Schema, contract_dir: &Path) -> Result<()> {
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

    let module_name = schema.collection.name();

    let package = package::Move {
        package: package::Package {
            name: module_name.clone(),
            version: "0.0.1".to_string(),
        },
        dependencies: package::Dependencies::new([
            (
                "Sui".to_string(),
                package::Dependency::new(
                    "https://github.com/MystenLabs/sui.git".to_string(),
                    SUI_PACKAGE_COMMIT_TEST.to_string(),
                )
                .subdir(
                    "crates/sui-framework/packages/sui-framework".to_string(),
                ),
            ),
            (
                "Originmate".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/originmate.git".to_string(),
                    ORIGINMATE_PACKAGE_COMMIT_TEST.to_string(),
                ),
            ),
            (
                "NftProtocol".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/nft-protocol".to_string(),
                    PROTOCOL_PACKAGE_COMMIT_TEST.to_string(),
                )
                .subdir("contracts/nft_protocol".to_string()),
            ),
            (
                "Launchpad".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/nft-protocol".to_string(),
                    PROTOCOL_PACKAGE_COMMIT_TEST.to_string(),
                )
                .subdir("contracts/launchpad".to_string()),
            ),
            (
                "LiquidityLayerV1".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/nft-protocol".to_string(),
                    PROTOCOL_PACKAGE_COMMIT_TEST.to_string(),
                )
                .subdir("contracts/liquidity_layer_v1".to_string()),
            ),
        ]),
        addresses: package::Addresses::new([(
            module_name.clone(),
            "0x0".to_string(),
        )]),
    };

    let mut buffer = String::new();
    let mut ser = toml::Serializer::pretty(&mut buffer);
    package.serialize(&mut ser).map_err(|err| {
        anyhow!(
            r#"Could not write package file "{}": {err}"#,
            package_path.display()
        )
    })?;

    package_file.write_all(buffer.as_bytes()).map_err(|err| {
        anyhow!(
            r#"Could not write package file "{}": {err}"#,
            package_path.display()
        )
    })?;

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
    contract_dir: &PathBuf,
) -> Result<CollectionState> {
    let wallet_ctx = rust_sdk::utils::get_context().await?;

    let gas_coin =
        rust_sdk::utils::get_coin_ref(&coin::get_max_coin(&wallet_ctx).await?);

    let collection_state = publish::publish_contract(
        &wallet_ctx,
        contract_dir,
        gas_coin,
        gas_budget as u64,
    )
    .await?;

    Ok(collection_state)
}
