use crate::storage::{aws::AWSConfig, pinata::PinataConfig};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Storage {
    Aws(AWSConfig),
    Pinata(PinataConfig),
}
