use clap::Parser;
use gutenberg::Schema;
use std::ffi::OsStr;
use std::fs::File;
use std::path::Path;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[arg(short, long)]
    demo: bool,
    input_config_path: String,
    output_dir: String,
}

fn main() {
    let Cli {
        demo,
        input_config_path,
        output_dir,
    } = Cli::parse();

    let config_path_parsed = Path::new(&input_config_path);
    let output_dir_parsed = Path::new(&output_dir);
    generate_contract(!demo, &config_path_parsed, &output_dir_parsed);
}

fn generate_contract(is_full: bool, config_path: &Path, output_dir: &Path) {
    let schema = assert_schema(config_path);

    // Create main contract directory
    let package_name = schema.package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    // Create `Move.toml`
    let move_file = File::create(contract_dir.join("Move.toml")).unwrap();
    schema
        .write_move_toml(move_file)
        .expect("Could not write `Move.toml`");

    let module_name = schema.nft().module_name();
    let module_file =
        File::create(sources_dir.join(format!("{module_name}.move"))).unwrap();
    schema
        .write_move(is_full, module_file)
        .expect("Could not write Move module");
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
                std::process::exit(1);
            }
        },
        "json" => match serde_json::from_reader::<_, Schema>(config) {
            Ok(schema) => schema,
            Err(err) => {
                eprintln!(
                    "Could not parse `{path}` due to {err}",
                    path = path.display()
                );
                std::process::exit(1);
            }
        },
        _ => {
            eprintln!("Extension {extension} not supported");
            std::process::exit(1);
        }
    }
}
