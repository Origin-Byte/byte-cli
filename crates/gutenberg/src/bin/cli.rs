use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    GenerateContract {
        input_config_path: String,
        output_dir: String,
    },

    GenerateTests,
}
