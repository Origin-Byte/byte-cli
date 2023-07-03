use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};

mod manifest;
pub mod models;
mod schema;

pub use manifest::{generate_manifest, write_manifest};
pub use schema::Schema;

/// Used to return all files for loading contract
///
/// TODO: Stream this directly into consumer to avoid transferring bulk strings
/// around
#[derive(Debug)]
pub struct ContractFile {
    path: PathBuf,
    content: String,
}

impl ContractFile {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }

    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn write<W: Write>(&self, mut target: W) -> Result<(), io::Error> {
        target.write_all(self.content.as_bytes())
    }

    pub fn write_to_file(&self, output_dir: &Path) -> Result<(), io::Error> {
        let path = output_dir.join(self.path.as_path());
        let mut file = File::create(path)?;
        self.write(&mut file)
    }
}

pub fn generate_contract_dir(schema: &Schema, output_dir: &Path) -> PathBuf {
    // Create main contract directory
    let package_name = schema.package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    contract_dir
}

// Consume `Schema` since we are modifying it
pub fn generate_contract(
    mut schema: Schema,
    is_demo: bool,
) -> Vec<ContractFile> {
    if is_demo {
        schema.enforce_demo();
    }

    let mut files = Vec::new();
    files.push(schema.write_move());

    files
}

/// Normalizes text into valid type name
pub fn normalize_type(type_name: &str) -> String {
    deunicode(type_name)
        .chars()
        .filter_map(|char| match char {
            '_' => Some('_'),
            '-' => Some('_'),
            ' ' => Some('_'),
            char => char.is_ascii_alphanumeric().then_some(char),
        })
        .collect()
}

/// De-unicodes and removes all unknown characters
pub fn deunicode(unicode: &str) -> String {
    deunicode::deunicode_with_tofu(unicode, "")
}
