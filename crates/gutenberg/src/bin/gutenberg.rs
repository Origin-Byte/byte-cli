use clap::{Parser, Subcommand};
use gutenberg::Schema;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Generate {
        input_config_path: String,
        output_dir: String,
    },

    GenerateTests,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate {
            input_config_path,
            output_dir,
        } => {
            let config_path_parsed = Path::new(&input_config_path);
            let output_dir_parsed = Path::new(&output_dir);
            generate_contract(&config_path_parsed, &output_dir_parsed);
        }
        Commands::GenerateTests => generate_tests(),
    }
}

fn generate_contract(config_path: &Path, output_dir: &Path) {
    let schema = assert_schema(config_path);

    // Create main contract directory
    let package_name = schema.collection().package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    // Create `Move.toml`
    let move_file = File::create(contract_dir.join("Move.toml")).unwrap();
    schema
        .write_move_toml(move_file)
        .expect("Could not write `Move.toml`");

    let module_file =
        File::create(sources_dir.join(format!("{package_name}.move"))).unwrap();
    schema
        .write_move(module_file)
        .expect("Could not write Move module");
}

fn generate_tests() {
    let scenarios_path = Path::new("./tests/scenarios");
    let modules_path = Path::new("./tests/packages");

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
fn assert_schema(path: &Path) -> Schema {
    let config = File::open(path).unwrap();
    let extension =
        path.extension().and_then(OsStr::to_str).unwrap_or_default();

    match extension {
        "yaml" => match serde_yaml::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = path.display()
                );
                std::process::exit(2);
            }
        },
        "json" => match serde_json::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = path.display()
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
