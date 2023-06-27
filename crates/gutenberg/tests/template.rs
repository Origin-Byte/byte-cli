//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;
use regex::Regex;

/// Test that template is up to date
#[test]
fn template() {
    let config = include_str!("./../template.json");
    remove_comments_and_parse(config).unwrap();
}

fn remove_comments_and_parse(
    content: &str,
) -> Result<Schema, serde_json::Error> {
    // Regex pattern to match single-line comments starting with "//"
    let comment_pattern = Regex::new(r"(?m)^\s*//.*$").unwrap();
    let json_without_comments = comment_pattern.replace_all(&content, "");

    serde_json::from_str(&json_without_comments)
}
