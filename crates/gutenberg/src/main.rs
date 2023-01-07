use gutenberg::prelude::*;

use gumdrop::Options;

use std::fs;
use std::path::PathBuf;

#[derive(Debug, Options)]
struct Opt {
    #[options(free)]
    config: PathBuf,
    #[options(help = "output file path")]
    output: Option<PathBuf>,
    #[options(help = "print help message")]
    help: bool,
}

fn main() -> Result<(), GutenError> {
    let opt = Opt::parse_args_default_or_exit();

    let f = fs::File::open(opt.config)?;

    let schema: Schema = match serde_yaml::from_reader(f) {
        Ok(schema) => schema,
        Err(err) => {
            eprintln!("Gutenberg could not generate smart contract due to");
            eprintln!("{}", err);
            std::process::exit(2);
        }
    };

    // If output file was not specified we prepare build directory for user to
    // publish directly after invoking gutenberg
    if opt.output.is_none() {
        fs::create_dir_all("./build")?;
        fs::File::create("./build/Move.toml")?;
        fs::copy("./examples/packages/Move.toml", "./build/Move.toml")?;
    }

    // Identify final output path and create intermediate directories
    let output_file = opt.output.unwrap_or_else(|| {
        PathBuf::from(&format!(
            "./build/sources/{}.move",
            &schema.module_name().to_string()
        ))
    });

    if let Some(p) = output_file.parent() {
        fs::create_dir_all(p)?;
    }

    let mut f = fs::File::create(output_file)?;
    if let Err(err) = schema.write_move(&mut f) {
        eprintln!("{err}");
    }

    Ok(())
}
