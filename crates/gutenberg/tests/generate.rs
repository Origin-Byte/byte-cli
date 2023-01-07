//! Integration tests directly check the generated examples in the parent directory

use gutenberg::schema::Schema;
use std::fs::{self, File};

/// Check that all examples have correct schema
#[test]
fn example_schema() {
    fs::read_dir("./examples")
        .unwrap()
        .map(Result::unwrap)
        // Filter out packages directory
        .filter(|f| f.file_type().unwrap().is_file())
        .map(|dir| {
            let config = File::open(dir.path()).unwrap();
            assert_schema(config);
        })
        .collect::<()>()
}

#[test]
fn suimarines() {
    assert_equal("suimarines.yaml", "suimarines.move");
}

#[test]
fn suitraders() {
    assert_equal("suitraders.yaml", "suitraders.move");
}

fn setup(config: &str, expected: &str) -> (File, String) {
    let config = File::open(format!("./examples/{config}")).unwrap();
    let expected =
        fs::read_to_string(format!("./examples/packages/sources/{expected}"))
            .unwrap();

    (config, expected)
}

/// Asserts that the config file has correct schema
fn assert_schema(config: File) -> Schema {
    serde_yaml::from_reader::<_, Schema>(config).unwrap()
}

/// Asserts that the generated file matches the expected output
fn assert_equal(config: &str, expected: &str) {
    let (config, expected) = setup(config, expected);

    let mut output = Vec::new();
    assert_schema(config).write_move(&mut output).unwrap();
    let output = String::from_utf8(output).unwrap();

    pretty_assertions::assert_eq!(output, expected);
}
