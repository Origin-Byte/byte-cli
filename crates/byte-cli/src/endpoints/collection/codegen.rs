use anyhow::{anyhow, Result};
use console::style;
use gutenberg_types::Schema;
use std::fs::File;
use std::path::Path;

/// Parses the configuration file to return a Schema object.
///
/// # Arguments
/// * `config_file` - A reference to a Path representing the configuration file.
///
/// # Returns
/// Result containing the Schema object or an error if parsing fails.
pub fn parse_config(config_file: &Path) -> Result<Schema, anyhow::Error> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find configuration file "{}": {err}
Call `byte-cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Schema>(file).map_err(|err|anyhow!(r#"Could not parse configuration file "{}": {err}
Call `byte-cli init-collection-config to initialize the configuration file again."#, config_file.display()))
}

pub async fn gen_contract(contract_dir: &Path, schema: &Schema) -> Result<()> {
    gutenberg::generate_project_with_flavors(
        schema,
        contract_dir,
        Some(String::from("1.3.0")), // TODO: This should not be hardcoded
    )?;

    println!(
        "{} Contract successfully generated: {:?}",
        style("DONE").green().bold(),
        contract_dir
    );

    Ok(())
}
