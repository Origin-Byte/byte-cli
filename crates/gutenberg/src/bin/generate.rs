//# Generates test contracts

use gutenberg::Schema;

use std::fs::{self, File};
use std::path::Path;

fn main() {
    let scenarios_path = Path::new("./tests/scenarios");

    for entry in std::fs::read_dir(scenarios_path)
        .unwrap()
        .filter_map(Result::ok)
    {
        let config_path = entry.path();

        let expected_file = config_path.file_stem().unwrap().to_str().unwrap();

        let config_extension = config_path
            .extension()
            .expect("Could not identify path extension")
            .to_str()
            .unwrap();

        let mut output = Vec::new();

        let config = File::open(config_path.as_path()).unwrap();
        assert_schema(config_path.as_path(), config, config_extension)
            .write_move(&mut output)
            .expect("Could not write move file");

        fs::write(&expected_file, output).unwrap();
    }
}

/// Asserts that the config file has correct schema
fn assert_schema(config_path: &Path, config: File, extension: &str) -> Schema {
    match extension {
        "yaml" => match serde_yaml::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = config_path.display()
                );
                std::process::exit(2);
            }
        },
        "json" => match serde_json::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = config_path.display()
                );
                std::process::exit(2);
            }
        },
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(2);
        }
    }
}
