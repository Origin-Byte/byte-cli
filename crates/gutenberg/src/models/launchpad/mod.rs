use crate::{InitArgs, MoveInit};
use gutenberg_types::models::launchpad::Launchpad;
pub mod listing;
pub mod market;

impl MoveInit for Launchpad {
    // TODO: To deprecate. The creation of listins will be done at runtime
    // in atomic transactions instead of being bundled up in the init funciton
    fn write_move_init(&self, _args: InitArgs) -> String {
        let code = self
            .listings
            .0
            .iter()
            .map(|listing| listing.write_move_init(InitArgs::None))
            .collect::<Vec<_>>();

        code.join("\n")
    }
}
