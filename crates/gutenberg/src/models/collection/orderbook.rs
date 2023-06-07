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

    pub fn write_move_defs(&self, type_name: &str) -> String {
        // TODO: Conditional on importing LiquidityLayer V1
        match self {
            Orderbook::Unprotected => String::new(), // do nothing
            Orderbook::Protected => format!(
                "

    // Protected orderbook functions
    public entry fun enable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<{type_name}, sui::sui::SUI>,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(false, false, false),
        );
    }}

    public entry fun disable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<{type_name}, sui::sui::SUI>,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(true, true, true),
        );
    }}"
            ),
            Orderbook::None => String::new(), // do nothing
        }
    }
}
