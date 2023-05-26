use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    pub type_name: String,
}

impl NftData {
    pub fn new(type_name: String) -> Self {
        Self { type_name }
    }

    pub fn write_struct(&self) -> String {
        let type_name = &self.type_name;
        format!(
            "
    struct {type_name} has key, store {{
        id: sui::object::UID,
        name: std::string::String,
        description: std::string::String,
        url: sui::url::Url,
        attributes: nft_protocol::attributes::Attributes,
    }}"
        )
    }
}
