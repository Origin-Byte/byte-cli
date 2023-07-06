use crate::{InitArgs, MoveInit};
use gutenberg_types::models::collection::{Tag, Tags};

impl MoveInit for Tag {
    fn write_move_init(&self, _args: Option<InitArgs>) -> String {
        match self {
            Tag::Custom(tag) => format!(
                "
        std::vector::push_back(&mut tags, std::string::utf8(b\"{tag}\"));"
            ),
            tag => {
                let function_name = tag.function_name();
                format!(
                    "
        std::vector::push_back(&mut tags, nft_protocol::tags::{function_name}());"
                )
            }
        }
    }
}

impl MoveInit for Tags {
    /// Generates Move code to push tags to a Move `vector` structure
    fn write_move_init(&self, _args: Option<InitArgs>) -> String {
        let mut code = String::from(
            "

        let tags = std::vector::empty();",
        );

        for tag in self.0.iter() {
            code.push_str(&tag.write_move_init(None));
        }

        code
    }
}
