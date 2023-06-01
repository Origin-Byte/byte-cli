use std::{fs::File, path::PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use derive_more::Display;
use sui_sdk::types::base_types::ObjectID;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct CollectionState {
    pub contract: Option<ObjectType>,
    pub collection: Option<ObjectType>,
    pub mint_cap: Option<ObjectType>,
    pub warehouses: Vec<ObjectType>,
}

#[derive(Debug, Serialize, Deserialize, Display)]
pub enum ObjectType {
    Package(ObjectID),
    Collection(ObjectID),
    MintCap(ObjectID),
    Warehouse(ObjectID),
    BpsRoyaltyStrategy(ObjectID),
    PolicyCap(ObjectID),
    Policy(ObjectID),
    TransferPolicy(ObjectID),
    TransferPolicyCap(ObjectID),
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
