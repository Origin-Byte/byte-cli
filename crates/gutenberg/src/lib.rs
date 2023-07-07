mod manifest;
mod models;
mod schema;

use anyhow::{anyhow, Result};
use gutenberg_types::{
    models::{collection::CollectionData, nft::Fields},
    Schema,
};
use manifest::write_toml_string;
pub use manifest::{write_manifest};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

pub trait MoveInit {
    fn write_move_init(&self, args: InitArgs) -> String;
}

pub trait MoveDefs {
    fn write_move_defs(&self, args: DefArgs) -> String;
}

pub trait WriteMove {
    fn write_move(&self) -> ContractFile;
}

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

pub trait MoveTests {
    fn write_move_tests(&self, args: TestArgs) -> String;
}

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

pub fn generate_contract_dir(schema: &Schema, output_dir: &Path) -> PathBuf {
    // Create main contract directory
    let package_name = schema.package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    contract_dir
}

// Consume `Schema` since we are modifying it
pub fn generate_contract_with_schema(
    mut schema: Schema,
    is_demo: bool,
) -> Vec<ContractFile> {
    if is_demo {
        schema.enforce_demo();
    }

    let mut files = Vec::new();
    files.push(schema.write_move());

    files
}

pub fn generate_contract_with_path(
    is_demo: bool,
    config_path: &Path,
    output_dir: &Path,
) -> Result<(), io::Error> {
    let schema = assert_schema(config_path);
    let contract_dir = generate_contract_dir(&schema, output_dir);

    write_manifest(schema.package_name(), &contract_dir)?;
    generate_contract_with_schema(schema, is_demo)
        .into_iter()
        .try_for_each(|file| file.write_to_file(&contract_dir))?;

    Ok(())
}

/// Asserts that the config file has correct schema
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

/// Normalizes text into valid type name
pub fn normalize_type(type_name: &str) -> String {
    deunicode(type_name)
        .chars()
        .filter_map(|char| match char {
            '_' => Some('_'),
            '-' => Some('_'),
            ' ' => Some('_'),
            char => char.is_ascii_alphanumeric().then_some(char),
        })
        .collect()
}

/// De-unicodes and removes all unknown characters
pub fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}

// TODO:
pub fn generate_contract(
    schema: &Schema,
    contract_dir: &Path,
    version: Option<String>,
) -> Result<()> {
    let (main_registry, test_registry) =
        package_manager::get_program_registries()?;

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
    fs::create_dir_all(contract_dir.join("flavours/"))?;

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
    fs::copy(main_toml_path, contract_dir.join("Move.toml"))?;

    // Write Move contract
    let move_path = &sources_dir.join(format!("{module_name}.move"));


    // let mut move_file = File::create(move_path).map_err(|err| {
    //     anyhow!(r#"Could not create "{}": {err}"#, move_path.display())
    // })?;

    // write!(&mut move_file, "{}", schema.write_move()).with_context(|| {
    //     anyhow!(r#"Could not write Move contract "{}""#, move_path.display())
    // })?;

    // Maybe add
    // let mut files = Vec::new();
    // files.push(schema.write_move());

    Ok(())
}
