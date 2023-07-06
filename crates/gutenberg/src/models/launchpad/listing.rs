use crate::{InitArgs, MoveInit};
use gutenberg_types::models::launchpad::listing::Listing;

impl MoveInit for Listing {
    fn write_move_init(&self, _args: Option<InitArgs>) -> String {
        let mut string = String::new();

        string.push_str(&format!(
            "
        let listing = nft_protocol::listing::new(
            @{admin},
            @{receiver},
            ctx,
        );

        let venue_id =
            nft_protocol::listing::create_venue(&mut listing, ctx);
",
            admin = self.admin,
            receiver = self.receiver,
        ));

        for market in self.markets.iter() {
            string.push_str(&market.write_move_init(None));
        }

        string.push_str(self.share());

        string
    }
}
