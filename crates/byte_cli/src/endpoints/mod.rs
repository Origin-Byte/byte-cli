use std::{fs, path::PathBuf};

use byte_cli::prelude::CliError;
use gutenberg::Schema;

pub mod config_collection;
pub mod config_upload;
pub mod deploy_assets;
pub mod deploy_contract;
pub mod mint_nfts;

pub fn try_read_config(path: &str) -> Result<Schema, CliError> {
    let path_2 = PathBuf::from(path);
    let f = fs::File::open(path);

    let schema = match f {
        Ok(file) => match serde_json::from_reader(file) {
            Ok(schema) => Ok(schema),
            Err(err) => Err(err),
        },
        Err(err) => Ok(Schema::default()),
    }?;

    Ok(schema)
}
