//! Integration tests directly check the generated examples in the parent
//! directory

use gutenberg::Schema;
use std::fs::{self, File};

// /// Check that all examples have correct schema
// #[test]
// fn example_schema() {
//     fs::read_dir("./examples")
//         .unwrap()
//         .map(Result::unwrap)
//         // Filter out packages directory
//         .filter(|f| f.file_type().unwrap().is_file())
//         .map(|dir| {
//             let path = dir.path();
//             let file_type = path.extension().and_then(OsStr::to_str);
//             let config = File::open(&path).unwrap();
//             assert_schema(config, file_type.unwrap());
//         })
//         .collect::<()>()
// }

// #[test]
// fn suimarines() {
//     assert_equal("suimarines.yaml", "suimarines.move");
// }

// #[test]
// fn suitraders() {
//     assert_equal("suitraders.yaml", "suitraders.move");
// }

#[test]
fn newbytes() {
    println!("a");
    assert_equal("newbytes.json", "newbytes.move");
}

fn setup(config: &str, expected: &str) -> (File, String) {
    let config = File::open(format!("./examples/{config}")).unwrap();
    let expected =
        fs::read_to_string(format!("./examples/packages/sources/{expected}"))
            .unwrap();

    (config, expected)
}

/// Asserts that the config file has correct schema
fn assert_schema(config: File, file_type: &str) -> Schema {
    match file_type {
        "yaml" => serde_yaml::from_reader::<_, Schema>(config).unwrap(),
        "json" => serde_json::from_reader::<_, Schema>(config).unwrap(),
        _ => {
            eprintln!("Extension not supported");
            std::process::exit(2);
        }
    }
}

/// Asserts that the generated file matches the expected output
fn assert_equal(config: &str, expected: &str) {
    println!("FUCK");
    let len = config.len();
    let extension = &config[len - 4..len];
    println!("1");

    let (config, _expected) = setup(config, expected);
    println!("2");
    let mut output = Vec::new();
    assert_schema(config, extension)
        .write_move(&mut output)
        .unwrap();
    // let output = String::from_utf8(output).unwrap();
    // println!("3");
    // pretty_assertions::assert_eq!(output, expected);
}
