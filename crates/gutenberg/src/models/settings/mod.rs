pub mod composability;
pub mod minting;
pub mod royalties;
pub mod tags;

pub use composability::Composability;
pub use minting::MintPolicies;
pub use royalties::RoyaltyPolicy;
pub use tags::Tags;

use serde::{Deserialize, Serialize};

use super::collection::CollectionData;

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub tags: Option<Tags>,               // Done
    pub royalties: Option<RoyaltyPolicy>, // Done
    pub mint_policies: MintPolicies,
    pub composability: Option<Composability>,
    #[serde(default)]
    pub loose: bool,
}

impl Settings {
    pub fn new(
        tags: Option<Tags>,
        royalties: Option<RoyaltyPolicy>,
        mint_policies: MintPolicies,
        composability: Option<Composability>,
        loose: bool,
    ) -> Settings {
        Settings {
            tags,
            royalties,
            mint_policies,
            composability,
            loose,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_none()
            && self.royalties.is_none()
            && self.mint_policies.is_empty()
            && self.composability.is_none()
            && !self.loose
    }

    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = Option::Some(tags);
    }

    pub fn set_royalties(&mut self, royalties: RoyaltyPolicy) {
        self.royalties = Option::Some(royalties);
    }

    pub fn set_mint_policies(&mut self, policies: MintPolicies) {
        self.mint_policies = policies;
    }

    pub fn set_composability(&mut self, composability: Composability) {
        self.composability = Option::Some(composability);
    }

    pub fn set_loose(&mut self, is_loose: bool) {
        self.loose = is_loose;
    }

    pub fn write_feature_domains(&self, collection: &CollectionData) -> String {
        let mut code = String::new();

        if let Some(_tags) = &self.tags {
            code.push_str(self.write_tags().as_str());
        }

        if let Some(_royalties) = &self.royalties {
            code.push_str(self.write_royalties().as_str());
        }

        if let Some(_composability) = &self.composability {
            code.push_str(self.write_composability().as_str());
        }

        if self.loose {
            code.push_str(self.write_loose(collection).as_str());
        }

        code
    }

    pub fn write_transfer_fns(&self, receiver: Option<&String>) -> String {
        let receiver = match receiver {
            Some(address) => {
                if address == "sui::tx_context::sender(ctx)" {
                    address.clone()
                } else {
                    format!("@{address}")
                }
            }
            None => "sui::tx_context::sender(ctx)".to_string(),
        };

        let mut code = format!(
            "
        sui::transfer::transfer(mint_cap, {receiver});
        sui::transfer::share_object(collection);\n"
        );

        if self.loose {
            code.push_str(
                format!(
                    "        sui::transfer::transfer(templates, {receiver});"
                )
                .as_str(),
            )
        }

        code
    }

    pub fn write_tags(&self) -> String {
        self.tags
            .as_ref()
            .expect("No collection tags setup found")
            .write_tags_vec()
    }

    pub fn write_royalties(&self) -> String {
        self.royalties
            .as_ref()
            .expect("No collection royalties setup found")
            .write_domain()
    }

    pub fn write_composability(&self) -> String {
        self.composability
            .as_ref()
            .expect("No collection composability setup found")
            .write_domain()
    }

    pub fn write_loose(&self, collection: &CollectionData) -> String {
        format!(
            "\n        let templates = templates::new_templates<{witness}>(
                ctx,
            );\n",
            witness = collection.witness_name(),
        )
    }

    pub fn write_type_declarations(&self) -> String {
        match &self.composability {
            Some(composability) => composability.write_types(),
            None => "".to_string(),
        }
    }
}
