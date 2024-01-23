use anyhow::{Context, Result};
use package_manager::{
    package::{Flavor, PackageRegistry},
    toml::{self as move_toml, MoveToml},
    version::Version,
};
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

/// Writes a manifest file for a given package.
///
/// # Arguments
///
/// * `package_name` - The name of the package.
/// * `flavor` - The network flavor, whether `Mainnet` or `Testnet`.
/// * `contract_dir` - The directory where the manifest file will be written.
/// * `main_registry` - The main package registry.
/// * `version` - Optional specific version of the package.
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure.
pub fn write_manifest(
    package_name: String,
    flavor: Flavor,
    contract_dir: &Path,
    main_registry: &PackageRegistry,
    version: Option<Version>,
) -> Result<()> {
    let manifest = write_toml_string(
        package_name.as_str(),
        flavor,
        &version,
        &main_registry,
    )?;

    let path = contract_dir.join("Move.toml");
    let mut file = File::create(path)?;
    file.write_all(manifest.as_bytes())?;

    Ok(())
}

/// Writes a manifest file with flavours for a given package.
///
/// This function generates two TOML files for main and test environments and
/// places them in a 'flavours' directory.
///
/// # Arguments
///
/// * `package_name` - The name of the package.
/// * `flavor` - The network flavor, whether `Mainnet` or `Testnet`.
/// * `contract_dir` - The directory where the manifest files will be written.
/// * `main_registry` - The main package registry.
/// * `test_registry` - The test package registry.
/// * `version` - Optional specific version of the package.
///
/// # Returns
///
/// Returns a `Result<()>` indicating success or failure.
pub fn write_manifest_with_flavours(
    package_name: String,
    contract_dir: &Path,
    main_registry: &PackageRegistry,
    test_registry: &PackageRegistry,
    version: Option<Version>,
) -> Result<()> {
    let main_toml_string = write_toml_string(
        package_name.as_str(),
        Flavor::Mainnet,
        &version,
        &main_registry,
    )?;

    let test_toml_string = write_toml_string(
        package_name.as_str(),
        Flavor::Testnet,
        &version,
        &test_registry,
    )?;

    let flavours_path = contract_dir.join("flavours/");
    fs::create_dir_all(&flavours_path).with_context(|| {
        format!(
            r#"Could not create "{path}""#,
            path = flavours_path.display()
        )
    })?;

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

/// Generates a TOML string for a module's manifest.
///
/// This function constructs the TOML representation of a module's manifest,
/// including dependencies and versions.
///
/// # Arguments
///
/// * `module_name` - The name of the module.
/// * `flavor` - The network flavor, whether `Mainnet` or `Testnet`.
/// * `version` - Optional specific version for the module.
/// * `registry` - The package registry used for resolving dependencies.
///
/// # Returns
///
/// Returns a `Result<String>` containing the TOML string or an error.
pub fn write_toml_string(
    module_name: &str,
    flavor: Flavor,
    version: &Option<Version>,
    registry: &PackageRegistry,
) -> Result<String> {
    let mut move_toml = match version {
        Some(version) => MoveToml::get_toml(
            module_name,
            flavor,
            registry,
            &[
                String::from("NftProtocol"),
                String::from("Launchpad"),
                String::from("LiquidityLayerV1"),
            ],
            &[String::from("Sui"), String::from("Originmate")],
            version,
        )?,
        None => MoveToml::get_toml_latest(
            module_name,
            flavor,
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
