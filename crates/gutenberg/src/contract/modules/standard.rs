use serde::{Deserialize, Serialize};

use super::Module;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StringMod();

impl Module for StringMod {
    fn import(&self) -> String {
        "use std::string::{Self, String};".to_string()
    }
}

impl StringMod {
    pub fn to_string_param(text: &str) -> String {
        format!("string::utf8(b\"{}\"),", text)
    }
}
