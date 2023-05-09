use serde::{Deserialize, Serialize};

pub enum RequestType {
    Transfer,
    Borrow,
    //Withdraw, TODO
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPolicies {
    pub transfer: bool,
    pub borrow: bool,
}

impl RequestPolicies {
    pub fn new(transfer: bool, borrow: bool) -> RequestPolicies {
        RequestPolicies { transfer, borrow }
    }

    pub fn is_empty(&self) -> bool {
        !self.transfer && !self.borrow
    }

    pub fn write_policies(&self, type_name: &String) -> String {
        let mut request_policies = String::new();

        if self.transfer {
            request_policies.push_str(&format!(
                "
        let (transfer_policy, transfer_policy_cap) =
            ob_request::transfer_request::init_policy<{type_name}>(&sui::package::publisher, ctx);

        nft_protocol::royalty_strategy_bps::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        nft_protocol::transfer_allowlist::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );"
            ));
        }
        if self.borrow {
            request_policies.push_str(&format!(
                "
        let (borrow_policy, borrow_policy_cap) =
            ob_request::borrow_request::init_policy<{type_name}>(&sui::package::publisher, ctx);\n"
            ));
        }

        request_policies
    }
}
