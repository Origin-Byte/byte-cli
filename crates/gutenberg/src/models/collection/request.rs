use serde::{Deserialize, Serialize};

use crate::models::nft::NftData;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RequestPolicies {
    #[serde(default)]
    transfer: bool,
    #[serde(default)]
    withdraw: bool,
    #[serde(default)]
    borrow: bool,
}

impl Default for RequestPolicies {
    /// Not providing any request policies by default is reasonable as it does
    /// not have any implications that the creator might be forced to consider
    /// with regards to creating safe policies.
    ///
    /// Should other features require policies, they will be implemented in a
    /// safe way.
    fn default() -> Self {
        Self {
            transfer: false,
            withdraw: false,
            borrow: false,
        }
    }
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

    pub fn has_transfer(&self) -> bool {
        self.transfer
    }

    pub fn has_withdraw(&self) -> bool {
        self.withdraw
    }

    pub fn has_borrow(&self) -> bool {
        self.borrow
    }

    pub fn write_move_init(&self, nft_data: &NftData) -> String {
        let type_name = nft_data.type_name();

        let mut request_policies = String::new();

        if self.transfer {
            request_policies.push_str(&format!(
                "

        let (transfer_policy, transfer_policy_cap) = ob_request::transfer_request::init_policy<{type_name}>(
            &publisher, ctx,
        );
        nft_protocol::royalty_strategy_bps::enforce(&mut transfer_policy, &transfer_policy_cap);
        nft_protocol::transfer_allowlist::enforce(&mut transfer_policy, &transfer_policy_cap);"
            ));
        }

        if self.borrow || nft_data.requires_borrow() {
            request_policies.push_str(&format!(
                "

        let (borrow_policy, borrow_policy_cap) = ob_request::borrow_request::init_policy<{type_name}>(
            &publisher, ctx,
        );"
            ));
        }

        if self.withdraw || nft_data.requires_withdraw() {
            request_policies.push_str(&format!(
                "

        let (withdraw_policy, withdraw_policy_cap) = ob_request::withdraw_request::init_policy<{type_name}>(
            &publisher, ctx,
        );"
            ));

            // When `NftData` requires a withdraw policy we must be careful to
            // protect it such that a malicious actor may not withdraw
            // arbitrarily
            if !self.withdraw {
                request_policies.push_str(&format!(
                    "
        ob_request::request::enforce_rule_no_state<ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>, Witness>(
            &mut withdraw_policy, &withdraw_policy_cap,
        );"
                ));
            }
        }

        request_policies
    }
}
