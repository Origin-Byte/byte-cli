use anyhow::{anyhow, Result};
use gutenberg::{package, Schema};
use serde::Serialize;
use std::io::Write;
use std::path::Path;
use std::{
    fs::{self, File},
    process::{Command, Stdio},
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

pub fn generate_contract(schema: &Schema, contract_dir: &Path) -> Result<()> {
    println!("Generating contract");

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

    let module_name = schema.module_name();

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
                    "0ef3625".to_string(),
                )
                .subdir("crates/sui-framework".to_string()),
            ),
            (
                "Originmate".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/originmate.git".to_string(),
                    "99c0e38".to_string(),
                ),
            ),
            (
                "NftProtocol".to_string(),
                package::Dependency::new(
                    "https://github.com/Origin-Byte/nft-protocol".to_string(),
                    "ebf3e4f".to_string(),
                ),
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
    })
}

pub fn publish_contract(
    gas_budget: usize,
    client_config: Option<&Path>,
    schema: &Schema,
    contract_dir: &Path,
) -> Result<()> {
    let gas_budget = gas_budget.to_string();
    let mut args =
        vec!["client", "publish", "--json", "--gas-budget", &gas_budget];
    if let Some(client_config) = client_config {
        args.append(&mut vec![
            "client.config",
            client_config.to_str().unwrap(),
        ]);
    }

    let package_path = contract_dir.to_str().unwrap();
    args.push(package_path);

    let module_name = schema.module_name();

    // Could not pull sui-sdk as a dependency so yolo
    let output = Command::new("sui")
        .args(args)
        .stdout(Stdio::inherit())
        .output()
        .map_err(|err| {
            anyhow!(r#"Could not publish module "{module_name}": {err}"#)
        })?;

    println!("{output:?}");

    Ok(())
}
