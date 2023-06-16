use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Orderbook {
    None,
    Unprotected,
    Protected,
}

impl Default for Orderbook {
    /// No orderbook by default is a safe choice
    fn default() -> Self {
        Orderbook::None
    }
}

impl Orderbook {
    pub fn write_move_init(&self, type_name: &str) -> String {
        match self {
            Orderbook::Unprotected => format!(
                "

        liquidity_layer_v1::orderbook::create_unprotected<{type_name}, sui::sui::SUI>(
            delegated_witness, &transfer_policy, ctx,
        );"
            ),
            Orderbook::Protected => format!(
                "

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<{type_name}, sui::sui::SUI>(
            delegated_witness, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);"
            ),
            Orderbook::None => String::new(), // do nothing
        }
    }
}
