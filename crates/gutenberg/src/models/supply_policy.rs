use serde::{Deserialize, Serialize};

use crate::err::GutenError;

#[derive(Debug, Deserialize, Serialize)]
pub enum SupplyPolicy {
    #[serde(alias = "unlimited")]
    Unlimited,
    #[serde(alias = "limited")]
    Limited {
        limit: u64,
        #[serde(default)]
        frozen: bool,
    },
    #[serde(alias = "undefined")]
    Undefined,
}

impl Default for SupplyPolicy {
    fn default() -> Self {
        SupplyPolicy::Undefined
    }
}

impl SupplyPolicy {
    pub fn new(
        input: &str,
        limit: Option<u64>,
        frozen: Option<bool>,
    ) -> Result<SupplyPolicy, GutenError> {
        match input {
            "Unlimited" => Ok(SupplyPolicy::Unlimited),
            "Limited" => Ok(SupplyPolicy::Limited {
                limit: limit.unwrap(),
                frozen: frozen.unwrap(),
            }),
            other => Err(GutenError::UnsupportedSettings(format!(
                "Unsupported supply policy `{other}`",
            ))),
        }
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, SupplyPolicy::Undefined)
    }

    pub fn write_domain(&self) -> String {
        match self {
            SupplyPolicy::Unlimited => format!(
                "
        nft_protocol::supply_domain::delegate_unregulated_and_transfer(
            &mint_cap,
            &mut collection,
            ctx,
        );\n"
            ),
            SupplyPolicy::Limited { limit, frozen } => format!(
                "
        nft_protocol::supply_domain::regulate(
            delegated_witness,
            &mut collection,
            {limit},
            {frozen},
        );

        nft_protocol::supply_domain::delegate_and_transfer(
            &mint_cap,
            &mut collection,
            {limit},
            ctx,
        );\n"
            ),
            SupplyPolicy::Undefined => "".to_string(),
        }
    }
}
