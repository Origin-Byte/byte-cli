use crate::{InitArgs, MoveInit};
use gutenberg_types::models::collection::Supply;

impl MoveInit for Supply {
    fn write_move_init(&self, _args: InitArgs) -> String {
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
