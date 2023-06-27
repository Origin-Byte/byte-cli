use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
// TODO: Add some kind of "optimistic" tracking that maintains supply tracking
// without requiring a mutable `Collection` parameter
pub enum Supply {
    Untracked,
    Tracked,
    Enforced(u64),
}

impl Default for Supply {
    /// Untracked is a reasonable default as it does not introduce any
    /// additional invariants that must be maintained by the user.
    fn default() -> Self {
        Supply::Untracked
    }
}

impl Supply {
    pub fn untracked() -> Self {
        Supply::Untracked
    }

    pub fn tracked() -> Self {
        Supply::Tracked
    }

    pub fn enforced(supply: u64) -> Self {
        Supply::Enforced(supply)
    }

    /// Whether supply needs to be tracked on the `Collection` level
    pub fn requires_collection(&self) -> bool {
        !matches!(self, Supply::Untracked)
    }

    pub fn write_move_domain(&self) -> String {
        let supply = match self {
            Supply::Untracked => return String::new(),
            Supply::Tracked => u64::MAX,
            Supply::Enforced(supply) => *supply,
        };

        format!(
            "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::supply::new(
                delegated_witness, {supply}, false,
            )
        );"
        )
    }

    pub fn write_move_increment() -> &'static str {
        "

        let supply = nft_protocol::supply::borrow_domain_mut(
            nft_protocol::collection::borrow_uid_mut(delegated_witness, collection),
        );

        nft_protocol::supply::increment(delegated_witness, supply, 1);"
    }

    pub fn write_move_decrement() -> &'static str {
        "

        let supply = nft_protocol::supply::borrow_domain_mut(
            nft_protocol::collection::borrow_uid_mut(delegated_witness, collection),
        );

        nft_protocol::supply::decrement(delegated_witness, supply, 1);
        nft_protocol::supply::decrease_supply_ceil(delegated_witness, supply, 1);"
    }
}
