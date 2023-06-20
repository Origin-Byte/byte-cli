pub mod err;
pub mod models;
mod schema;

pub use schema::Schema;

/// Normalizes text into valid type name
pub fn normalize_type(type_name: &str) -> String {
    deunicode(type_name)
        .chars()
        .filter_map(|char| match char {
            '_' => Some('_'),
            '-' => Some('_'),
            ' ' => Some('_'),
            char => char.is_ascii_alphanumeric().then_some(char),
        })
        .collect()
}

/// De-unicodes and removes all unknown characters
pub fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}
