pub mod composability;
pub mod minting;
pub mod orderbook;
pub mod request;
pub mod royalties;

pub use composability::Composability;
pub use minting::MintPolicies;
pub use orderbook::Orderbook;
pub use request::RequestPolicies;
pub use royalties::RoyaltyPolicy;

use serde::{Deserialize, Serialize};

use super::{collection::CollectionData, nft::NftData};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub royalties: Option<RoyaltyPolicy>,
    pub mint_policies: MintPolicies,
    pub request_policies: RequestPolicies,
    pub composability: Option<Composability>,
    pub orderbook: Orderbook,
}

impl Settings {
    pub fn new(
        royalties: Option<RoyaltyPolicy>,
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
        composability: Option<Composability>,
        orderbook: Orderbook,
    ) -> Settings {
        Settings {
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

    pub fn write_request_policies(&self, nft_data: &NftData) -> String {
        self.request_policies.write_policies(nft_data)
    }

    pub fn write_transfer_fns(&self, nft_data: &NftData) -> String {
        let mut code = String::new();

        code.push_str(
            "

        // Setup Transfers
        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(collection);",
        );

        if self.request_policies.transfer {
            code.push_str(
                "

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);"
            );
        }

        if self.request_policies.withdraw || nft_data.requires_withdraw() {
            code.push_str(
                "

        sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(withdraw_policy);"
            );
        }

        if self.request_policies.borrow || nft_data.requires_borrow() {
            code.push_str(
                "

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);"
            );
        }

        code
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
