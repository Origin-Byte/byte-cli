pub mod cli;
pub mod consts;
pub mod prelude;

use crate::prelude::*;
use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() {}

async fn run() -> Result<()> {
    let _cli = Cli::parse();

    Ok(())
}
