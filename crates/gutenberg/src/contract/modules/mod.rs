pub mod protocol;
pub use protocol::{
    CollectionMod, ComposableNftMod, CreatorsMod, DisplayInfoMod, MintCapMod,
    NftMod, RoyaltiesMod, RoyaltyMod, WarehouseMod, WitnessMod,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Display();

impl Display {
    pub fn write_display(type_name: &String) -> String {
        format!("let display = sui::display::new<{type_name}>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b\"name\"), std::string::utf8(b\"{{name}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"description\"), std::string::utf8(b\"{{description}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"image_url\"), std::string::utf8(b\"{{url}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"attributes\"), std::string::utf8(b\"{{attributes}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"tags\"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);
        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));"
    )
    }
}
