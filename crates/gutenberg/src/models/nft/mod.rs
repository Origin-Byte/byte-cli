pub mod burn;
pub mod dynamic;

use burn::Burn;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    type_name: String,
    burn: Burn,
    dynamic: bool,
}

impl NftData {
    pub fn type_name(&self) -> &String {
        &self.type_name
    }

    pub fn burn(&self) -> &Burn {
        &self.burn
    }

    pub fn dynamic(&self) -> bool {
        self.dynamic
    }

    /// Returns whether NFT requires withdraw policy to be created
    pub fn requires_withdraw(&self) -> bool {
        self.burn.is_permissionless()
    }

    /// Returns whether NFT requires borrow policy to be created
    pub fn requires_borrow(&self) -> bool {
        self.dynamic
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

    pub fn write_burn_fns(&self) -> String {
        self.burn.write_burn_fns(self.type_name())
    }

    pub fn write_burn_tests(&self, witness_type: &String) -> String {
        self.burn.write_burn_tests(self.type_name(), witness_type)
    }
}
