use crate::err::GutenError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Nft {
    nft_type: NftType,
    supply_policy: bool,
    fields: Fields,
    mint_strategy: MintStrategy,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Display {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Url {
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Attributes {
    keys: String,
    values: String,
}
