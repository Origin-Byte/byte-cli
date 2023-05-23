use std::{
    // env,
    fs::File,
    path::{Path, PathBuf},
};

use crate::prelude::CliError;
use anyhow::anyhow;
use gutenberg::schema::{Schema, SchemaBuilder};

use rust_sdk::collection_state::CollectionState;
use serde::Serialize;
use uploader::writer::Storage;

pub fn try_read_schema(path_buf: &PathBuf) -> Result<SchemaBuilder, CliError> {
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

pub fn write_schema(
    schema: &SchemaBuilder,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    schema.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}

pub fn write_schema_(
    schema: &Schema,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    schema.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}

pub fn read_uploader(path_buf: &PathBuf) -> Result<Storage, CliError> {
    let file = File::open(path_buf)?;
    let uploader = serde_json::from_reader(file)?;

    Ok(uploader)
}

pub fn write_uploader(
    uploader: &Storage,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create configuration file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    uploader.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}

pub fn write_collection_state(
    state: &CollectionState,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create collection state file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    state.serialize(ser).map_err(|err| {
        anyhow!(
            r#"Could not write configuration file "{}": {err}"#,
            output_file.display()
        )
    })
}

pub fn get_path_buf(path: &str) -> PathBuf {
    let mut path_buf = PathBuf::from(path);
    path_buf.push("config");
    path_buf.set_extension("json");

    path_buf
}
