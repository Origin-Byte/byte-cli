use crate::GutenError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Serialize)]
pub struct Launchpad {
    admin: String,
    candy_machines: Vec<CandyMachine>,
}

pub struct CandyMachine {
    markets: Vec<Market>,
    mint_style: MintStyle,
}

pub enum MintStyle {
    AheadOfTime,
    JustInTime,
}
