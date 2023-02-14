use std::process::Output;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SuiOutput {
    pub certificate: Certificate,
    pub effects: Effects,
    // timestamp_ms: bool,
    // parsed_data: bool,
}

impl SuiOutput {
    pub fn from_output(output: &Output) -> Self {
        let out = format!("{:?}", output);
        let sui_output: SuiOutput = serde_json::from_str(&out).unwrap();

        sui_output
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Certificate {
    pub transaction_digest: String,
    pub data: Data,
    pub tx_signature: String,
    pub auth_sign_info: AuthSignInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Data {
    // pub transactions: Transactions,
    pub sender: String,
    pub gas_payment: GasPayment,
    pub gas_price: u64,
    pub gas_budget: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasPayment {
    object_id: String,
    version: u64,
    digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthSignInfo {
    epoch: String,
    signature: u64,
    signers_map: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Effects {
    status: Status,
    gas_used: GasUsed,
    transaction_digest: String,
    created: Vec<Object>,
    mutated: Vec<Object>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GasUsed {
    computation_cost: u64,
    storage_cost: u64,
    storage_rebate: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Object {
    owner: u64,
    reference: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    object_owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference {
    object_id: String,
    version: u64,
    digest: String,
}
