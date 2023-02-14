use anyhow::{anyhow, Result};
use gutenberg::{package, Schema};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;
use std::process::Output;
use std::{
    fs::{self, File},
    process::{Command, Stdio},
};

use crate::models::sui_output::SuiOutput;
use rust_sdk::publish;

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

pub async fn publish_contract(
    gas_budget: usize,
    client_config: Option<&Path>,
    schema: &Schema,
    contract_dir: &Path,
) -> Result<()> {
    let client = publish::get_client().await.unwrap();
    let keystore = publish::get_keystore().await.unwrap();

    publish::publish_contract(&client, &keystore).await;

    // let gas_budget = gas_budget.to_string();
    // let mut args = vec![
    //     "client",
    //     "publish",
    //     "--json",
    //     "--gas-budget",
    //     &gas_budget,
    //     "--skip-dependency-verification",
    // ];
    // if let Some(client_config) = client_config {
    //     args.append(&mut vec![
    //         "client.config",
    //         client_config.to_str().unwrap(),
    //     ]);
    // }

    // let package_path = contract_dir.to_str().unwrap();
    // args.push(package_path);

    // let module_name = schema.module_name();

    // // Could not pull sui-sdk as a dependency so yolo
    // let output = Command::new("sui")
    //     .args(args)
    //     .stdout(Stdio::inherit())
    //     .output()
    //     .map_err(|err| {
    //         anyhow!(r#"Could not publish module "{module_name}": {err}"#)
    //     })?;

    // println!("{output:?}");

    // let sui_output = SuiOutput::from_output(&output);

    // println!("YOOOOOH \n {sui_output:?}");

    Ok(())
}

// pub fn print_certificate(output: Output) {
//     // ----- Certificate ----
//     // Transaction Hash: TransactionDigest(2WmrfxMgQoFUEXGi9wcKvQXqqaXFF2qKyvcagH1KEb6W)
//     // Transaction Signature: AA==@9gqzLGPc7GrLUoo/wIr1OVPSP5/ZzbKnLYMJNlcp1bThdwm9mQVxbOWCvG4Bk8eiiM+Ri5O8m2ENG8Cg7CZGDA==@xmYzcPMFXID/19jg9KxYpxL27OiKwgPFkAA5a/m0rTc=
//     // Signed Authorities Bitmap: RoaringBitmap<[0, 1, 2]>
//     // Transaction Kind : Publish
//     // Sender: 0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143
//     // Gas Payment: Object ID: 0x0e2905e94f7b4a815099a1088d18d37508fe56b8, version: 0xa64, digest: o#45SuCHJCltvTn3A2IK7BohG/0YivU8OGnZ3l3t5OYss=
//     // Gas Price: 1
//     // Gas Budget: 60000
//     println!("----- Certificate ----");
//     println!("Transaction Hash: TransactionDigest({})", output)
// }
