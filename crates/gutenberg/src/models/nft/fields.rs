use crate::{InitArgs, MoveInit};
use gutenberg_types::models::nft::{Field, FieldType};

impl MoveInit for Field {
    fn write_move_init(&self, _args: Option<InitArgs>) -> String {
        let field_name = self.name();
        match self.field_type() {
            FieldType::String => String::from(""), // TODO: This was changed from None, double check correctness
            FieldType::Url => String::from("sui::url::new_unsafe_from_bytes({field_name})"),
            FieldType::Attributes => String::from("nft_protocol::attributes::from_vec({field_name}_keys, {field_name}_values)"),
        }
    }
}

impl MoveInit for FieldType {
    fn write_move_init(&self, _args: Option<InitArgs>) -> String {
        match self {
            FieldType::String => String::from("std::string::String"),
            FieldType::Url => String::from("sui::url::Url"),
            FieldType::Attributes => {
                String::from("nft_protocol::attributes::Attributes")
            }
        }
    }
}
