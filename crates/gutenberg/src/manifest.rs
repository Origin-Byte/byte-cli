use anyhow::{anyhow, Context, Result};
use gutenberg_types::models::address::Address;
use package_manager::{
    package::{GitPath, Package, PackageRegistry},
    toml::{self as move_toml, MoveToml},
    version::Version,
};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{self, Write},
    path::Path,
    str::FromStr,
};

pub fn generate_manifest(package_name: String) -> MoveToml {
    MoveToml::new(
        Package::new(package_name.clone(), Version::new(0, 0, 0), None),
        BTreeMap::from([
            (
                "Launchpad".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/launchpad".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
            (
                "NftProtocol".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/nft_protocol".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
            (
                "LiquidityLayerV1".to_string(),
                GitPath::new(
                    "https://github.com/Origin-Byte/nft-protocol.git"
                        .to_string(),
                    Some("contracts/liquidity_layer_v1".to_string()),
                    "v1.3.0-mainnet".to_string(),
                ),
            ),
        ]),
        BTreeMap::from([(package_name, Address::zero())]),
    )
}

pub fn write_manifest(
    package_name: String,
    contract_dir: &Path,
) -> Result<(), io::Error> {
    let manifest = generate_manifest(package_name);

    let path = contract_dir.join("Move.toml");
    let mut file = File::create(path)?;
    file.write_all(manifest.to_string().unwrap().as_bytes())?;

    Ok(())
}

// TODO!
// pub fn write_manifest_with_flavours(
//     package_name: String,
//     contract_dir: &Path,
// ) -> Result<(), io::Error> {
//     // let manifest = generate_manifest(package_name);

//     let flavours_path = contract_dir.join("flavours/");
//     fs::create_dir_all(&flavours_path).with_context(|| {
//         format!(
//             r#"Could not create "{path}""#,
//             path = flavours_path.display()
//         )
//     })?;

//     let main_toml_path = contract_dir.join("flavours/Move-main.toml");
//     let mut main_toml_file =
//         File::create(&main_toml_path).with_context(|| {
//             format!(
//                 r#"Could not create "{path}""#,
//                 path = main_toml_path.display()
//             )
//         })?;

//     Ok(())
// }

// TODO:
// pub fn generate_contract(
//     schema: &Schema,
//     contract_dir: &Path,
//     version: Option<String>,
// ) -> Result<(), io::Error> {
//     let (main_registry, test_registry) =
//         package_manager::get_program_registries()?;

//     let sources_dir = &contract_dir.join("sources");
//     let _ = fs::remove_dir_all(sources_dir);
//     fs::create_dir_all(sources_dir).map_err(|err| {
//         anyhow!(
//             r#"Could not create directory "{}": {err}"#,
//             sources_dir.display()
//         )
//     })?;

//     // Write Move.toml
//     // Create the directory if it doesn't exist
//     fs::create_dir_all(contract_dir.join("flavours/"))?;

//     let main_toml_path = &contract_dir.join("flavours/Move-main.toml");
//     let mut mail_toml_file = File::create(main_toml_path).map_err(|err| {
//         anyhow!(
//             r#"Could not create Move.toml "{}": {err}"#,
//             main_toml_path.display()
//         )
//     })?;

//     // Write Move-test.toml
//     let test_toml_path = &contract_dir.join("flavours/Move-test.toml");
//     let mut test_toml_file = File::create(test_toml_path).map_err(|err| {
//         anyhow!(
//             r#"Could not create Move-test.toml "{}": {err}"#,
//             test_toml_path.display()
//         )
//     })?;

//     let module_name = schema.package_name();

//     let main_toml_string =
//         write_toml_string(module_name.as_str(), &version, main_registry)?;

//     let test_toml_string =
//         write_toml_string(module_name.as_str(), &version, test_registry)?;

//     // Output
//     mail_toml_file.write_all(main_toml_string.as_bytes())?;
//     test_toml_file.write_all(test_toml_string.as_bytes())?;

//     // Copy Main Move.toml
//     fs::copy(main_toml_path, contract_dir.join("Move.toml"))?;

//     // Write Move contract
//     let move_path = &sources_dir.join(format!("{module_name}.move"));
//     let mut move_file = File::create(move_path).map_err(|err| {
//         anyhow!(r#"Could not create "{}": {err}"#, move_path.display())
//     })?;

//     write!(&mut move_file, "{}", schema.write_move()).with_context(|| {
//         anyhow!(r#"Could not write Move contract "{}""#, move_path.display())
//     })?;

//     Ok(())
// }

pub fn write_toml_string(
    module_name: &str,
    version: &Option<String>,
    registry: &PackageRegistry,
) -> Result<String> {
    let mut move_toml = match version {
        Some(version) => MoveToml::get_toml(
            module_name,
            registry,
            &[
                String::from("NftProtocol"),
                String::from("Launchpad"),
                String::from("LiquidityLayerV1"),
            ],
            &[String::from("Sui"), String::from("Originmate")],
            &Version::from_str(version.as_str())?,
        )?,
        None => MoveToml::get_toml_latest(
            module_name,
            registry,
            &[
                String::from("NftProtocol"),
                String::from("Launchpad"),
                String::from("LiquidityLayerV1"),
            ],
            &[String::from("Sui"), String::from("Originmate")],
        )?,
    };

    move_toml.sanitize_output();

    let mut toml_string = toml::to_string_pretty(&move_toml)?;
    toml_string = move_toml::add_vertical_spacing(toml_string.as_str());

    Ok(toml_string)
}
