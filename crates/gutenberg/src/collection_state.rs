use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CollectionState {
    pub mint_cap: String,
    pub warehouses: Vec<String>,
}

impl CollectionState {
    pub fn try_read_config(path_buf: &PathBuf) -> Result<Self> {
        let f = File::open(path_buf);

        let schema = match f {
            Ok(file) => match serde_json::from_reader(file) {
                Ok(schema) => Ok(schema),
                Err(err) => Err(anyhow!("The following error has occurred while reading objects.json: {},", err)),
            },
            Err(err) => Err(anyhow!("The following error has occurred while reading objects.json: {},", err)),
        }?;

        Ok(schema)
    }
}
