//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;

use regex::Regex;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::Path;

#[test]
fn scenarios() {
    let scenarios_path = Path::new("./tests/scenarios");

    let files = std::fs::read_dir(scenarios_path)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            path.is_file().then_some(path)
        });

    for path in files {
        let expected_file =
            path.file_stem().unwrap().to_str().unwrap().to_string();

        let mut output = Vec::new();

        assert_schema(&path)
            .write_move(&mut output)
            .expect("Could not write move file");

        let output = String::from_utf8(output).unwrap();

        let expected = assert_expected(&format!("{expected_file}.move"));
        pretty_assertions::assert_eq!(output, expected);
    }
}

/// Test that template is up to date
#[test]
fn template() {
    let config = include_str!("./../template.json");
    remove_comments_and_parse(config).unwrap();
}

fn assert_expected(expected: &str) -> String {
    let expected_path = Path::new("./tests/packages/sources").join(expected);

    match fs::read_to_string(&expected_path) {
        Ok(contract) => contract,
        Err(_) => {
            panic!(
                "Could not find expected source path: {expected_path}
Run `gutenberg generate-tests` to generate the missing scenario source.",
                expected_path = expected_path.display()
            );
        }
    }
}

/// Asserts that the config file has correct schema
fn assert_schema(path: &Path) -> Schema {
    let config = File::open(path).unwrap();
    let extension =
        path.extension().and_then(OsStr::to_str).unwrap_or_default();

    match extension {
        "yaml" => serde_yaml::from_reader::<_, Schema>(config).unwrap(),
        "json" => serde_json::from_reader::<_, Schema>(config).unwrap(),
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(2);
        }
    }
}

fn remove_comments_and_parse(
    content: &str,
) -> Result<Schema, serde_json::Error> {
    // Regex pattern to match single-line comments starting with "//"
    let comment_pattern = Regex::new(r"(?m)^\s*//.*$").unwrap();
    let json_without_comments = comment_pattern.replace_all(&content, "");

    serde_json::from_str(&json_without_comments)
}
