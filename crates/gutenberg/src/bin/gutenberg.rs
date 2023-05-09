use crate::cli::{Cli, Commands};
use clap::Parser;
use gutenberg::Schema;
use std::fs::File;
use std::path::Path;

pub mod cli;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input_config_path,
            output_dir,
        } => {
            println!(
                "Generating contract from {} to {}",
                input_config_path, output_dir
            );

            let config_path_parsed = Path::new(&input_config_path);
            let output_dir_parsed = Path::new(&output_dir);
            generate_contract(&config_path_parsed, &output_dir_parsed);
        }
        Commands::GenerateTests => generate_tests(),
    }
}

fn generate_contract(config_path: &Path, output_dir: &Path) {
    let expected_file = config_path.file_stem().unwrap().to_str().unwrap();
    let expected_file = format!("{expected_file}.move");

    let config_extension = match config_path.extension() {
        Some(extension) => extension.to_str().unwrap(),
        None => {
            eprintln!(
                "Could not identify file extension for configuration path: {}",
                config_path.display(),
            );
            return;
        }
    };

    let mut output = File::create(output_dir.join(expected_file)).unwrap();

    let config = File::open(config_path).unwrap();
    assert_schema(config_path, config, config_extension)
        .write_move(&mut output)
        .expect("Could not write move file");
}

fn generate_tests() {
    let scenarios_path = Path::new("./tests/scenarios");
    let modules_path = Path::new("./tests/packages/sources");

    let files = std::fs::read_dir(scenarios_path)
        .unwrap()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            path.is_file().then_some(entry)
        });

    for entry in files {
        let config_path = entry.path();
        generate_contract(&config_path, modules_path);
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
