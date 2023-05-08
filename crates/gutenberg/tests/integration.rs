//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

#[test]
fn scenarios() {
    let scenarios_path = Path::new("./tests/scenarios");

    for entry in std::fs::read_dir(scenarios_path)
        .unwrap()
        .filter_map(Result::ok)
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

        let expected_file = format!("{}.move", expected_file);
        let output_path =
            Path::new("./tests/packages/build").join(&expected_file);
        if let Err(e) = write_to_file(&output, &output_path) {
            eprintln!("Error writing to file: {}", e);
        } else {
            println!("Expected value written to {}", &output_path.display());
        }

        let expected_path =
            Path::new("./tests/packages/sources").join(&expected_file);

        let expected = assert_expected(&expected_path);
        // println!("Output size:\n{:#?}", output.len());
        // println!("Expected sizze:\n{:#?}", expected.len());

        pretty_assertions::assert_eq!(output, expected);
    }
}

fn write_to_file(expected: &str, file_path: &Path) -> std::io::Result<()> {
    let mut file = File::create(file_path)?;
    file.write_all(expected.as_bytes())?;
    Ok(())
}

fn assert_expected(expected_path: &Path) -> String {
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
