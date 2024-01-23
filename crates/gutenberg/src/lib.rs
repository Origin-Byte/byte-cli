mod manifest;
mod models;
mod schema;

use anyhow::{anyhow, Result};
use gutenberg_types::{
    models::{collection::CollectionData, nft::Fields},
    Schema,
};
pub use manifest::write_manifest;
use manifest::write_manifest_with_flavours;
use package_manager::{
    get_program_registry, package::Flavor, version::Version, Network,
};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

/// Trait for writing Move language contract's `init` function.
pub trait MoveInit {
    fn write_move_init(&self, args: InitArgs) -> String;
}

/// Trait for writing Move language contract's function definitions
pub trait MoveDefs {
    fn write_move_defs(&self, args: DefArgs) -> String;
}

/// Trait for writing Move language contract's tests
pub trait MoveTests {
    fn write_move_tests(&self, args: TestArgs) -> String;
}

/// Trait for writing Move language contract. It serves as a wrapper trait as it
/// orchestrates the high-level codegen
pub trait WriteMove {
    fn write_move(&self) -> ContractFile;
}

// Enums for arguments passed to the `MoveInit` trait...
pub enum InitArgs<'a> {
    MintCap {
        witness: &'a str,
        type_name: &'a str,
    },
    NftData {
        collection_data: &'a CollectionData,
    },
    CollectionData {
        type_name: &'a str,
    },
    Orderbook {
        type_name: &'a str,
    },
    None,
}

// Enums for arguments passed to the `MoveDefs` trait...
pub enum DefArgs<'a> {
    Burn {
        fields: &'a Fields,
        type_name: &'a str,
        requires_collection: bool,
        requires_listing: bool,
        requires_confirm: bool,
    },
    Dynamic {
        fields: &'a Fields,
        type_name: &'a str,
    },
    MintPolicies {
        fields: &'a Fields,
        type_name: &'a str,
        requires_collection: bool,
    },
    NftData {
        collection_data: &'a CollectionData,
    },
    None,
}

// Enums for arguments passed to the `MoveTests` trait...
pub enum TestArgs<'a> {
    Burn {
        fields: &'a Fields,
        type_name: &'a str,
        witness_name: &'a str,
        requires_collection: bool,
    },
    Dynamic {
        fields: &'a Fields,
        type_name: &'a str,
        witness_name: &'a str,
        requires_collection: bool,
    },
    MintPolicies {
        fields: &'a Fields,
        type_name: &'a str,
        witness_name: &'a str,
        requires_collection: bool,
    },
    Orderbook {
        fields: &'a Fields,
        type_name: &'a str,
        witness_name: &'a str,
        requires_collection: bool,
        requires_royalties: bool,
    },
    NftData {
        collection_data: &'a CollectionData,
    },
    None,
}

/// Used to return all files for loading contract
///
/// TODO: Stream this directly into consumer to avoid transferring bulk strings
/// around
#[derive(Debug)]
pub struct ContractFile {
    path: PathBuf,
    content: String,
}

impl ContractFile {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn write<W: Write>(&self, mut target: W) -> Result<(), io::Error> {
        target.write_all(self.content.as_bytes())
    }

    pub fn write_to_file(&self, output_dir: &Path) -> Result<(), io::Error> {
        let path = output_dir.join(self.path.as_path());
        let mut file = File::create(path)?;
        self.write(&mut file)
    }
}

/// Functions for generating and managing Move language contracts and projects.
///
/// Includes functions for generating contract directories, writing contracts,
/// asserting schema, normalizing type names, and generating projects with
/// flavors.
pub fn generate_contract_dir(schema: &Schema, output_dir: &Path) -> PathBuf {
    // Create main contract directory
    let package_name = schema.package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    contract_dir
}

/// Generates a contract with the given schema and optionally enforces demo
/// constraints.
///
/// # Arguments
/// * `schema` - A mutable reference to a Schema, representing the contract's
///   structure.
/// * `is_demo` - A boolean indicating if demo constraints should be enforced.
///
/// # Returns
/// A vector of `ContractFile` objects representing the generated contract
/// files.
///
/// # Functionality
/// - If `is_demo` is true, enforces demo constraints on the schema.
/// - Generates contract files based on the schema.
pub fn generate_contract_with_schema(schema: &Schema) -> Vec<ContractFile> {
    let mut files = Vec::new();
    files.push(schema.write_move());

    files
}

/// Generates a project with the given configuration and writes it to the
/// specified directory.
///
/// # Arguments
/// * `config_path` - Path to the configuration file.
/// * `flavor` - The network flavor, whether `Mainnet` or `Testnet`.
/// * `output_dir` - Path to the output directory for writing the project.
/// * `version` - Optional string representing the version of the project.
///
/// # Returns
/// Result indicating success or error.
///
/// # Functionality
/// - Asserts the schema from the configuration file.
/// - Generates the contract directory and writes the project's manifest.
/// - Generates and writes the contract files based on the schema.
pub fn generate_project(
    config_path: &Path,
    flavor: Flavor,
    output_dir: &Path,
    version: Option<String>,
) -> Result<()> {
    let schema = assert_schema(config_path);
    let contract_dir = generate_contract_dir(&schema, output_dir);

    let registry = get_program_registry(&Network::Mainnet)?;

    let version: Option<Version> = version.map(|s| s.parse().ok()).flatten();

    write_manifest(
        schema.package_name(),
        flavor,
        &contract_dir,
        &registry,
        version,
    )?;

    generate_contract_with_schema(&schema)
        .into_iter()
        .try_for_each(|file| file.write_to_file(&contract_dir))?;

    Ok(())
}

/// Asserts that the given configuration file has a correct schema.
///
/// # Arguments
/// * `path` - Path to the configuration file.
///
/// # Returns
/// The parsed `Schema` if successful.
///
/// # Functionality
/// - Opens and reads the configuration file.
/// - Parses the file based on its extension (either YAML or JSON) to a
///   `Schema`.
fn assert_schema(path: &Path) -> Schema {
    let config = File::open(path).unwrap();
    let extension =
        path.extension().and_then(OsStr::to_str).unwrap_or_default();

    match extension {
        "yaml" => match serde_yaml::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = path.display()
                );
                std::process::exit(1);
            }
        },
        "json" => match serde_json::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = path.display()
                );
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(1);
        }
    }
}

/// Generates a project with flavors, setting up the project structure and
/// writing the contract.
///
/// # Arguments
/// * `is_demo` - A boolean indicating if the project is a demo.
/// * `schema` - A mutable reference to the Schema.
/// * `contract_dir` - Path to the directory where the contract should be
///   written.
/// * `version` - Optional string representing the version of the project.
///
/// # Returns
/// Result indicating success or error.
///
/// # Functionality
/// - Sets up the project structure, including source directories.
/// - Writes the project's manifest with flavors.
/// - Generates and writes the contract files based on the schema.
pub fn generate_project_with_flavors(
    schema: &Schema,
    contract_dir: &Path,
) -> Result<()> {
    let (main_registry, test_registry) =
        package_manager::get_program_registries()?;

    let version = main_registry.get_latest_version("NftProtocol")?;

    let sources_dir = &contract_dir.join("sources");
    let _ = fs::remove_dir_all(sources_dir);
    fs::create_dir_all(sources_dir).map_err(|err| {
        anyhow!(
            r#"Could not create directory "{}": {err}"#,
            sources_dir.display()
        )
    })?;

    write_manifest_with_flavours(
        schema.package_name(),
        &contract_dir,
        &main_registry,
        &test_registry,
        Some(*version),
    )?;

    // Write Move contract
    generate_contract_with_schema(schema)
        .into_iter()
        .try_for_each(|file| file.write_to_file(&contract_dir))?;

    Ok(())
}
