//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;

use std::fs::{self, File};
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
        "json" => serde_json::from_reader::<_, Schema>(config).unwrap(),
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(2);
        }
    }
}
