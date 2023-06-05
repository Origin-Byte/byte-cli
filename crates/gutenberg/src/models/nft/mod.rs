pub mod burn;
pub mod dynamic;

use burn::Burn;
use dynamic::Dynamic;
use serde::{Deserialize, Serialize};

use super::collection::CollectionData;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    type_name: String,
    burn: Burn,
    dynamic: Dynamic,
}

impl NftData {
    pub fn new(type_name: String, burn: Burn, dynamic: bool) -> Self {
        NftData {
            type_name,
            burn,
            dynamic: dynamic.into(),
        }
    }

    pub fn type_name(&self) -> &String {
        &self.type_name
    }

    /// Returns whether NFT requires withdraw policy to be created
    pub fn requires_withdraw(&self) -> bool {
        self.burn.is_permissionless()
    }

    /// Returns whether NFT requires borrow policy to be created
    pub fn requires_borrow(&self) -> bool {
        self.dynamic.is_dynamic()
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

    pub fn write_dynamic_fns(&self) -> String {
        self.dynamic.write_dynamic_fns(self.type_name())
    }

    pub fn write_dynamic_tests(
        &self,
        collection_data: &CollectionData,
    ) -> String {
        self.dynamic
            .write_dynamic_tests(self.type_name(), collection_data)
    }

    pub fn write_burn_fns(&self, collection_data: &CollectionData) -> String {
        self.burn.write_burn_fns(&self.type_name(), collection_data)
    }

    pub fn write_burn_tests(&self, collection_data: &CollectionData) -> String {
        self.burn
            .write_burn_tests(self.type_name(), collection_data)
    }
}
