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
        generate_contract_with_path(demo, &config_path_parsed, &output_dir_parsed)
    {
        eprintln!("{err}");
    }
}