//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;

use regex::Regex;
use serde::de::{DeserializeOwned, Error};
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;

#[test]
fn scenarios() {
    let scenarios_path = Path::new("./tests/scenarios");

    for entry in std::fs::read_dir(scenarios_path)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
    {
        let path = entry.path();

        let expected_file =
            path.file_stem().unwrap().to_str().unwrap().to_string();

        let config_extension = path
            .extension()
            .expect("Could not identify path extension")
            .to_str()
            .unwrap()
            .to_string();

        let mut output = Vec::new();

        let config = File::open(path).unwrap();
        assert_schema(config, &config_extension)
            .write_move(&mut output)
            .expect("Could not write move file");

        let output = String::from_utf8(output).unwrap();

        let expected = assert_expected(&format!("{expected_file}.move"));
        pretty_assertions::assert_eq!(output, expected);
    }
}

fn assert_expected(expected: &str) -> String {
    let expected_path = Path::new("./tests/packages/sources").join(expected);
    fs::read_to_string(&expected_path).expect("Could not read expected")
}

/// Asserts that the config file has correct schema
fn assert_schema(config: File, extension: &str) -> Schema {
    match extension {
        "yaml" => serde_yaml::from_reader::<_, Schema>(config).unwrap(),
        "json" => remove_comments_and_parse::<Schema>(config).unwrap(),
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(2);
        }
    }
}

fn remove_comments_and_parse<T: DeserializeOwned>(
    mut file: File,
) -> Result<T, serde_json::Error> {
    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|_| serde_json::Error::custom("Failed to read file"))?;

    // Regex pattern to match single-line comments starting with "//"
    let comment_pattern = Regex::new(r"(?m)^\s*//.*$").unwrap();
    let json_without_comments = comment_pattern.replace_all(&content, "");

    serde_json::from_str(&json_without_comments)
}
