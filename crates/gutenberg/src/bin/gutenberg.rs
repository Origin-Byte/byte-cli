use clap::Parser;
use gutenberg::Schema;
use std::ffi::OsStr;
use std::fs::File;
use std::io;
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

    if let Err(err) =
        generate_contract(demo, &config_path_parsed, &output_dir_parsed)
    {
        eprintln!("{err}");
    }
}

fn generate_contract(
    is_demo: bool,
    config_path: &Path,
    output_dir: &Path,
) -> Result<(), io::Error> {
    let schema = assert_schema(config_path);
    let contract_dir = gutenberg::generate_contract_dir(&schema, output_dir);

    gutenberg::write_manifest(schema.package_name(), &contract_dir)?;
    gutenberg::generate_contract(schema, is_demo)
        .into_iter()
        .try_for_each(|file| file.write_to_file(&contract_dir))?;

    Ok(())
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
