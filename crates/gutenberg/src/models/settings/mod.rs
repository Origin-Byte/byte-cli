pub mod orderbook;
pub mod request;

pub use orderbook::Orderbook;
pub use request::RequestPolicies;

use serde::{Deserialize, Serialize};

use super::nft::NftData;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    request_policies: RequestPolicies,
    orderbook: Orderbook,
}

impl Settings {
    pub fn new(
        request_policies: RequestPolicies,
        orderbook: Orderbook,
    ) -> Settings {
        Settings {
            request_policies,
            orderbook,
        }
    }

    pub fn request_policies(&self) -> &RequestPolicies {
        &self.request_policies
    }

    pub fn orderbook(&self) -> &Orderbook {
        &self.orderbook
    }

    pub fn write_move_init(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut init_str = String::new();
        init_str.push_str(
            "

        let publisher = sui::package::claim(witness, ctx);",
        );
        init_str.push_str(&self.request_policies.write_policies(nft_data));
        init_str.push_str(&self.orderbook.write_move_init(type_name));
        init_str
    }

    pub fn write_move_defs(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut defs_str: String = String::new();
        defs_str.push_str(&self.orderbook.write_move_defs(type_name));
        defs_str
    }
}
