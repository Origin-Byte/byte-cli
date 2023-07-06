use crate::{InitArgs, MoveInit};
use gutenberg_types::models::nft::MintCap;

// TODO!
// impl MintCap {
//     /// Write MintCap instantiation with supply always limited to 100
//     pub fn write_move_demo_init(
//         &self,
//         witness: &str,
//         type_name: &str,
//     ) -> String {
//         format!("

//         let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
//             &witness, collection_id, 100, ctx
//         );
//         sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
//     }
// }

impl MoveInit for MintCap {
    /// Write MintCap instantiation
    fn write_move_init(&self, args: Option<InitArgs>) -> String {
        let (witness, type_name) = init_args(args);

        let mint_cap_str = match self.supply {
            Some(supply) => format!("

        let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
            &witness, collection_id, {supply}, ctx
        );"),
            None =>
            format!("

        let mint_cap = nft_protocol::mint_cap::new_unlimited<{witness}, {type_name}>(
            &witness, collection_id, ctx
        );")
        };

        format!("{mint_cap_str}
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
    }
}

fn init_args(args: Option<InitArgs>) -> (&str, &str) {
    // Add err handling
    let args = args.unwrap();

    match args {
        InitArgs::MintCap { witness, type_name } => (witness, type_name),
        _ => panic!("Incorrect InitArgs variant"),
    }
}
