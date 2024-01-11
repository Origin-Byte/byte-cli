use clap::Parser;
use gutenberg::generate_project;
use std::path::Path;

/// A struct representing command-line arguments.
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    /// Flag indicating whether to run in demo mode.
    #[arg(short, long)]
    demo: bool,
    /// Path to the input configuration file.
    input_config_path: String,
    /// Path to the output directory.
    output_dir: String,
}

/// The main entry point of the application.
fn main() {
    // Parsing command-line arguments into the Cli struct
    let Cli {
        demo,
        input_config_path,
        output_dir,
    } = Cli::parse();

    // Parsing the input and output paths from the command-line arguments
    let config_path_parsed = Path::new(&input_config_path);
    let output_dir_parsed = Path::new(&output_dir);

    // Attempt to generate a project based on the provided arguments
    // and handle potential errors.
    if let Err(err) = generate_project(
        demo,
        &config_path_parsed,
        &output_dir_parsed,
        Some(String::from("1.3.0")), // TODO: It should not be a fixed version string
    ) {
        eprintln!("{err}");
    }
}
