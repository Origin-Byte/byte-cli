use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    pub type_name: String,
}

impl NftData {
    pub fn write_struct(&self) -> String {
        let type_name = &self.type_name;
        format!(
            "struct {type_name} has key, store {{
                id: UID,
                name: String,
                description: String,
                url: Url,
                attributes: Attributes,
            }}"
        )
    }
}
