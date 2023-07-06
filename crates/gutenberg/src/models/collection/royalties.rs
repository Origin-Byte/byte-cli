use super::Address;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub trait MoveInit {
    fn write_move_init(&self) -> String;
}

impl MoveInit for RoyaltyPolicy {
    fn write_move_init(&self) -> String {
        match self {
            RoyaltyPolicy::Proportional {
                shares,
                collection_royalty_bps,
            } => {
                let mut creators_str = "

        let royalty_map = sui::vec_map::empty();"
                    .to_string();

                for share in shares {
                    creators_str.push_str(&format!(
                        "
        sui::vec_map::insert(&mut royalty_map, @{address}, {share});",
                        share = share.share_bps,
                        address = share.address
                    ));
                }

                let domain = format!(
                    "

        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            {collection_royalty_bps},
            ctx,
        );"
                );
                creators_str.push_str(&domain);
                creators_str
            }
        }
    }
}
