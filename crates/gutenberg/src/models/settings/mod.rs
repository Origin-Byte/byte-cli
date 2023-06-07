pub mod minting;
pub mod orderbook;
pub mod request;

pub use minting::MintPolicies;
pub use orderbook::Orderbook;
pub use request::RequestPolicies;

use serde::{Deserialize, Serialize};

use super::{collection::CollectionData, nft::NftData};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    mint_policies: MintPolicies,
    request_policies: RequestPolicies,
    orderbook: Orderbook,
}

impl Settings {
    pub fn new(
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
        orderbook: Orderbook,
    ) -> Settings {
        Settings {
            mint_policies,
            request_policies,
            orderbook,
        }
    }

    pub fn mint_policies(&self) -> &MintPolicies {
        &self.mint_policies
    }

    pub fn request_policies(&self) -> &RequestPolicies {
        &self.request_policies
    }

    pub fn orderbook(&self) -> &Orderbook {
        &self.orderbook
    }

    pub fn write_move_init(
        &self,
        nft_data: &NftData,
        collection_data: &CollectionData,
    ) -> String {
        let type_name = nft_data.type_name();
        let witness = collection_data.witness_name();

        let mut init_str = String::new();
        init_str
            .push_str(&self.mint_policies.write_move_init(&witness, type_name));
        init_str.push_str(
            "

        let publisher = sui::package::claim(witness, ctx);",
        );
        init_str.push_str(&self.request_policies.write_policies(nft_data));
        init_str.push_str(&self.orderbook.write_move_init(type_name));
        init_str
    }

    pub fn write_move_defs(
        &self,
        nft_data: &NftData,
        collection_data: &CollectionData,
    ) -> String {
        let type_name = nft_data.type_name();

        let mut defs_str: String = String::new();
        defs_str.push_str(
            &self
                .mint_policies
                .write_move_defs(nft_data, collection_data),
        );
        defs_str.push_str(&self.orderbook.write_move_defs(type_name));
        defs_str
    }
}
