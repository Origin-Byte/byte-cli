use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::models::project::Project;
use crate::prelude::CliError;
use anyhow::anyhow;
use gutenberg::schema::{Schema, SchemaBuilder};

use package_manager::{info::BuildInfo, move_lib::PackageMap, toml::MoveToml};
use rust_sdk::collection_state::CollectionState;
use serde::{de::DeserializeOwned, Serialize};
use uploader::writer::Storage;

impl LocalRead for Schema {}
impl LocalRead for Project {}
impl LocalRead for Storage {}
impl LocalRead for CollectionState {}
impl LocalRead for MoveToml {}
impl LocalRead for PackageMap {}
impl LocalRead for BuildInfo {}
impl LocalWrite for Schema {}
impl LocalWrite for Project {}
impl LocalWrite for Storage {}
impl LocalWrite for CollectionState {}
impl LocalWrite for SchemaBuilder {}
// impl LocalWrite for MoveToml {}

impl LocalRead for SchemaBuilder {
    fn read(path_buf: &PathBuf) -> Result<Self, CliError> {
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
            Err(_) => Ok(SchemaBuilder::default()),
        }?;

        Ok(schema)
    }
}

pub trait LocalRead: DeserializeOwned {
    fn read(path_buf: &PathBuf) -> Result<Self, CliError> {
        let file = File::open(path_buf)?;
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
    fn write(&self, output_file: &Path) -> Result<(), anyhow::Error> {
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
