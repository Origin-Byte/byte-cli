use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    pub type_name: String,
}

impl NftData {
    pub fn write_struct(&self) -> String {
        format!(
            "struct {} has key, store {{
                id: UID,
                name: String,
                description: String,
                url: Url,
                attributes: Attributes,
            }}",
            self.type_name
        )
    }
}
