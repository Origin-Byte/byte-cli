use serde::{Deserialize, Serialize};

use crate::models::nft::NftData;

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPolicies {
    pub transfer: bool,
    pub withdraw: bool,
    pub borrow: bool,
}

impl RequestPolicies {
    pub fn new(
        transfer: bool,
        withdraw: bool,
        borrow: bool,
    ) -> RequestPolicies {
        RequestPolicies {
            transfer,
            withdraw,
            borrow,
        }
    }

    pub fn write_policies(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut request_policies = String::new();

        if self.transfer {
            request_policies.push_str(&format!(
                "

        let (transfer_policy, transfer_policy_cap) =
            ob_request::transfer_request::init_policy<{type_name}>(&publisher, ctx);

        nft_protocol::royalty_strategy_bps::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        nft_protocol::transfer_allowlist::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );"
            ));
        }

        if self.borrow || nft_data.requires_borrow() {
            request_policies.push_str(&format!(
                "

        let (borrow_policy, borrow_policy_cap) =
            ob_request::borrow_request::init_policy<{type_name}>(&publisher, ctx);"
            ));
        }

        if self.withdraw || nft_data.requires_withdraw() {
            request_policies.push_str(&format!(
                "

        let (withdraw_policy, withdraw_policy_cap) =
            ob_request::withdraw_request::init_policy<{type_name}>(&publisher, ctx);"
            ))
        }

        request_policies
    }
}
