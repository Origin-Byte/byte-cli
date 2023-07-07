use anyhow::{anyhow, Context, Result};
use package_manager::{
    package::PackageRegistry,
    toml::{self as move_toml, MoveToml},
    version::Version,
};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    str::FromStr,
};

pub fn write_manifest(
    package_name: String,
    contract_dir: &Path,
    main_registry: &PackageRegistry,
) -> Result<()> {
    let manifest = write_toml_string(
        package_name.as_str(),
        &Some(String::from("1.3.0")),
        &main_registry,
    )?;

    let path = contract_dir.join("Move.toml");
    let mut file = File::create(path)?;
    file.write_all(manifest.as_bytes())?;

    Ok(())
}

pub fn write_manifest_with_flavours(
    package_name: String,
    contract_dir: &Path,
    main_registry: &PackageRegistry,
    test_registry: &PackageRegistry,
) -> Result<()> {
    let v = Version::from_str("1.3.0")?;

    let main_toml_string = write_toml_string(
        package_name.as_str(),
        &Some(String::from("1.3.0")),
        &main_registry,
    )?;

    let test_toml_string = write_toml_string(
        package_name.as_str(),
        &Some(String::from("1.3.0")),
        &test_registry,
    )?;

    let flavours_path = contract_dir.join("flavours/");
    fs::create_dir_all(&flavours_path).with_context(|| {
        format!(
            r#"Could not create "{path}""#,
            path = flavours_path.display()
        )
    })?;

    let path = contract_dir.join("Move.toml");

    let main_toml_path = contract_dir.join("flavours/Move-main.toml");
    let mut main_toml_file =
        File::create(&main_toml_path).with_context(|| {
            format!(
                r#"Could not create "{path}""#,
                path = main_toml_path.display()
            )
        })?;
    main_toml_file.write_all(main_toml_string.as_bytes())?;

    let test_toml_path = contract_dir.join("flavours/Move-test.toml");
    let mut test_toml_file =
        File::create(&test_toml_path).with_context(|| {
            format!(
                r#"Could not create "{path}""#,
                path = test_toml_path.display()
            )
        })?;
    test_toml_file.write_all(test_toml_string.as_bytes())?;

    // Copy Main Move.toml
    fs::copy(main_toml_path, contract_dir.join("Move.toml"))?;

    Ok(())
}

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
