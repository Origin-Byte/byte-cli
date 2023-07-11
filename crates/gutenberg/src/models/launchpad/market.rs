use crate::{InitArgs, MoveInit};
use gutenberg_types::models::launchpad::market::Market;

impl MoveInit for Market {
    fn write_move_init(&self, _args: InitArgs) -> String {
        match self {
            Market::FixedPrice {
                token,
                price,
                is_whitelisted,
            } => format!(
                "
        nft_protocol::fixed_price::create_market_on_listing<{token}>(
            &mut listing,
            venue_id,
            {is_whitelisted},
            {price},
            ctx,
        );
",
            ),
            Market::DutchAuction {
                token,
                reserve_price,
                is_whitelisted,
            } => format!(
                "
        nft_protocol::dutch_auction::create_market_on_listing<{token}>(
            &mut listing,
            venue_id,
            {is_whitelisted},
            {reserve_price},
            ctx,
        );
",
            ),
        }
    }
}
