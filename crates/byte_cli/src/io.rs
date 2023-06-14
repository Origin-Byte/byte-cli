use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use crate::models::project::Project;
use crate::prelude::CliError;
use anyhow::{anyhow, Result};
use git2::Repository;
use gutenberg::schema::{Schema, SchemaBuilder};

use package_manager::{info::BuildInfo, move_lib::PackageMap, toml::MoveToml};
use rust_sdk::{
    collection_state::CollectionState,
    metadata::{GlobalMetadata, StorableMetadata},
};
use serde::{de::DeserializeOwned, Serialize};
use tempfile::TempDir;
use uploader::writer::Storage;

impl LocalRead for Schema {}
impl LocalRead for Project {}
impl LocalRead for Storage {}
impl LocalRead for CollectionState {}
impl LocalRead for MoveToml {}
impl LocalRead for PackageMap {}
impl LocalRead for BuildInfo {}
impl LocalRead for GlobalMetadata {}
impl LocalWrite for Schema {}
impl LocalWrite for Project {}
impl LocalWrite for Storage {}
impl LocalWrite for CollectionState {}
impl LocalWrite for SchemaBuilder {}
impl LocalWrite for MoveToml {}
impl LocalWrite for StorableMetadata {}

impl LocalRead for SchemaBuilder {
    fn read_json(path_buf: &PathBuf) -> Result<Self, CliError> {
        let f = File::open(path_buf);

        let schema = match f {
            Ok(file) => match serde_json::from_reader(file) {
                Ok(schema) => Ok(schema),
                Err(err) => Err(err),
            },
            Err(_) => Ok(SchemaBuilder::default()),
        }?;

        Ok(schema)
    }

    fn read_yaml(path_buf: &PathBuf) -> Result<Self, CliError> {
        let f = File::open(path_buf);

        let schema = match f {
            Ok(file) => match serde_yaml::from_reader(file) {
                Ok(schema) => Ok(schema),
                Err(err) => Err(err),
            },
            Err(_) => {
                println!("Unable to find schema in: {:?}", path_buf);
                println!("Creating new schema");
                Ok(SchemaBuilder::default())
            }
        }?;

        Ok(schema)
    }
}

// TODO: Consider removing LocalRead in favor of DeserializeOwned
pub trait LocalRead: DeserializeOwned {
    fn read_json(path_buf: &PathBuf) -> Result<Self, CliError> {
        let file = File::open(path_buf)?;
        // TODO: Return a more telling error message
        let obj = serde_json::from_reader(file)?;

        Ok(obj)
    }
    fn read_yaml(path_buf: &PathBuf) -> Result<Self, CliError> {
        let file = File::open(path_buf)?;
        let obj = serde_yaml::from_reader(file)?;

        Ok(obj)
    }
}

pub trait LocalWrite: Serialize {
    fn write_json(&self, output_file: &Path) -> Result<(), anyhow::Error> {
        // Create the parent directories if they don't exist
        fs::create_dir_all(output_file.parent().unwrap())?;

        let file = File::create(output_file).map_err(|err| {
            anyhow!(
                r#"Could not create file "{}": {err}"#,
                output_file.display()
            )
        })?;

        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
        self.serialize(ser).map_err(|err| {
            anyhow!(
                r#"Could not write file "{}": {err}"#,
                output_file.display()
            )
        })
    }
}

pub fn get_project_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("project.json"))
}

pub fn get_schema_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("schema.json"))
}

pub fn get_upload_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("upload.json"))
}

pub fn get_assets_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "assets", None)
}

pub fn get_metadata_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "metadata", None)
}

pub fn get_upload_metadata(
    name: &str,
    path_opt: &Option<String>,
) -> (PathBuf, PathBuf) {
    let pre_upload =
        get_file_path(name, path_opt, "metadata", Some("pre-upload.json"));
    let post_upload =
        get_file_path(name, path_opt, "metadata", Some("post-upload.json"));

    (pre_upload, post_upload)
}

pub fn get_contract_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "contract", None)
}

pub fn get_toml_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "contract", Some("Move.toml"))
}

pub fn get_pakage_registry_paths() -> (TempDir, PathBuf) {
    let temp_dir =
        TempDir::new().expect("Failed to create temporary directory");

    let filename = PathBuf::from("registry-main.json");
    let mut registry_path = temp_dir.path().to_path_buf();
    registry_path.push(&filename);

    (temp_dir, registry_path)
}

pub fn get_build_info_path(
    name: &str,
    path_opt: &Option<String>,
) -> Result<PathBuf, CliError> {
    // Note: This code block assumes that there is only one folder
    // in the build folder, which is the case.PathBuf::from(Path::new(project_dir.as_str()));
    let mut build_info_path =
        get_file_path(name, path_opt, "contract/build", None);
    let mut paths = fs::read_dir(&build_info_path).unwrap();

    if let Some(path) = paths.next() {
        build_info_path = path?.path();
        build_info_path.push("BuildInfo.yaml");
    } else {
        return Err(CliError::from(anyhow!("Could not find path to BuildInfo.yaml. Call `sui move build` to compile the Sui Move package")));
    }

    Ok(build_info_path)
}

fn get_file_path(
    name: &str,
    path_opt: &Option<String>,
    folder: &str,
    filename: Option<&str>,
) -> PathBuf {
    let mut filepath: PathBuf;

    if let Some(path) = path_opt {
        filepath = PathBuf::from(Path::new(path.clone().as_str()));
    } else {
        filepath = dirs::home_dir().unwrap();
        filepath.push(format!(".byte/projects/{}", name));
    }

    filepath.push(format!("{}/", folder));

    if let Some(file) = filename {
        filepath.push(format!("{}", file));
    }

    filepath
}

pub fn write_json(
    vec: Vec<String>,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    // Create the parent directories if they don't exist
    fs::create_dir_all(output_file.parent().unwrap())?;

    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    vec.serialize(ser).map_err(|err| {
        anyhow!(r#"Could not write file "{}": {err}"#, output_file.display())
    })
}

pub fn get_program_registry() -> Result<PackageMap> {
    let (temp_dir, registry_path) = get_pakage_registry_paths();

    let url = "https://github.com/Origin-Byte/program-registry";

    let repo = match Repository::clone(url, temp_dir.path()) {
        Ok(repo) => repo,
        Err(e) => return Err(anyhow!("failed to clone: {}", e)),
    };

    if !repo.is_empty()? {
        println!("Fetched Program Registry successfully");
    } else {
        return Err(anyhow!(
            "Something went wrong while accessing the Program Registry"
        ));
    }

    let map = PackageMap::read_json(&registry_path)?;

    Ok(map)
}
