pub mod models;
mod schema;

pub use schema::Schema;

/// Normalizes a given type name into a valid Move language type name.
///
/// # Arguments
/// * `type_name` - The type name to normalize.
///
/// # Returns
/// A normalized type name suitable for use in Move.
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

/// Removes unicode characters and replaces them with ASCII equivalents.
///
/// # Arguments
/// * `unicode` - The unicode string to de-unicode.
///
/// # Returns
/// A de-unicoded ASCII string.
pub fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}
