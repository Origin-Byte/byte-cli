pub mod composability;
pub mod minting;
pub mod orderbook;
pub mod request;
pub mod royalties;
pub mod tags;

pub use composability::Composability;
pub use minting::MintPolicies;
pub use orderbook::Orderbook;
pub use request::RequestPolicies;
pub use royalties::RoyaltyPolicy;
pub use tags::Tags;

use serde::{Deserialize, Serialize};

use super::collection::CollectionData;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub tags: Option<Tags>,               // Done
    pub royalties: Option<RoyaltyPolicy>, // Done
    pub mint_policies: MintPolicies,
    pub request_policies: RequestPolicies,
    pub composability: Option<Composability>,
    pub orderbook: Orderbook,
}

impl Settings {
    pub fn new(
        tags: Option<Tags>,
        royalties: Option<RoyaltyPolicy>,
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
        composability: Option<Composability>,
        orderbook: Orderbook,
    ) -> Settings {
        Settings {
            tags,
            royalties,
            mint_policies,
            request_policies,
            composability,
            orderbook,
        }
    }

    pub fn write_feature_domains(
        &self,
        _collection: &CollectionData,
    ) -> String {
        let mut code = String::new();

        if let Some(_royalties) = &self.royalties {
            code.push_str(self.write_royalties().as_str());
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

        let mut code = String::new();

        code.push_str(&format!(
            "
        // Setup Transfers
        sui::transfer::public_transfer(publisher, {receiver});
        sui::transfer::public_transfer(mint_cap, {receiver});
        sui::transfer::public_transfer(allowlist_cap, {receiver});
        sui::transfer::public_share_object(allowlist);
        sui::transfer::public_share_object(collection);
        "
        ));

        if self.request_policies.transfer {
            code.push_str(&format!(
                "
        sui::transfer::public_transfer(transfer_policy_cap, {receiver});
        sui::transfer::public_share_object(transfer_policy);\n"
            ));
        }

        if self.request_policies.borrow {
            code.push_str(&format!(
                "
        sui::transfer::public_transfer(borrow_policy_cap, {receiver});
        sui::transfer::public_share_object(borrow_policy);\n"
            ));
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
            .write_strategy()
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
