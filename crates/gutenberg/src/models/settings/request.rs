use std::collections::HashSet;

use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};

use crate::err::GutenError;

pub enum RequestType {
    Transfer,
    Borrow,
    //Withdraw, TODO
}

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPolicies {
    #[serde(default)]
    pub transfer: bool,
    #[serde(default)]
    pub borrow: bool,
}

impl Default for RequestPolicies {
    fn default() -> Self {
        Self {
            transfer: false,
            borrow: false,
        }
    }
}

impl RequestPolicies {
    pub fn new(
        transfer: bool,
        borrow: bool,
    ) -> RequestPolicies {
        RequestPolicies { transfer, borrow }
    }

    pub fn is_empty(&self) -> bool {
        !self.transfer && !self.borrow
    }

    fn from_map(
        map: &Vec<(String, bool)>,
    ) -> Result<RequestPolicies, GutenError> {
        let mut field_struct = RequestPolicies::default();

        for (f, v) in map {
            match f.as_str() {
                "transfer" => {
                    field_struct.transfer = *v;
                    Ok(())
                }
                "borrow" => {
                    field_struct.borrow = *v;
                    Ok(())
                }
                other => Err(GutenError::UnsupportedSettings(format!(
                    "The Request policy provided `{}` is not supported",
                    other
                ))),
            }?;
        }

        Ok(field_struct)
    }

    pub fn fields() -> Vec<String> {
        let field_struct = RequestPolicies::default();
        let mut fields: Vec<String> = Vec::new();

        for (i, _) in field_struct.iter_fields().enumerate() {
            let field_name = field_struct.name_at(i).unwrap();

            fields.push(field_name.to_string());
        }
        fields
    }

    pub fn to_map(&self) -> Vec<(String, bool)> {
        let mut map: Vec<(String, bool)> = Vec::new();

        for (i, value) in self.iter_fields().enumerate() {
            let field_name = self.name_at(i).unwrap();
            let value_ = value.downcast_ref::<bool>().unwrap();
            map.push((field_name.to_string(), *value_));
        }
        map
    }

    pub fn write_policies(&self, type_name: &String) -> String {
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
        );
            "
            ));
        }
        if self.borrow {
            request_policies.push_str(&format!(
                "
        let (borrow_policy, borrow_policy_cap) =
            ob_request::borrow_request::init_policy<{type_name}>(&publisher, ctx);\n
            "
            ));
        }

        request_policies
    }
}
