use serde::{Deserialize, Serialize};

use super::Module;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StringMod();

impl Module for StringMod {
    fn import(&self, has_self: bool) -> String {
        if has_self {
            "    use std::string::{Self, String};\n".to_string()
        } else {
            "    use std::string::String;\n".to_string()
        }
    }
}

impl StringMod {
    pub fn to_string_param(text: &str) -> String {
        format!("std::string::utf8(b\"{}\"),", text)
    }
}
