use anyhow::{anyhow, Result};
// use git2::Repository;
use gutenberg_types::Schema;
use package_manager::{
    info::BuildInfo, package::PackageRegistry, toml::MoveToml, Network,
};
// use rust_sdk::metadata::{GlobalMetadata, StorableMetadata};
use serde::Serialize;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

pub fn get_project_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("project.json"))
}

pub fn get_schema_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("schema.json"))
}

pub fn get_upload_filepath(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "configs", Some("upload.json"))
}

pub fn get_assets_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "assets", None)
}

pub fn get_metadata_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "metadata", None)
}

pub fn get_upload_metadata(
    name: &str,
    path_opt: &Option<String>,
) -> (PathBuf, PathBuf) {
    let pre_upload =
        get_file_path(name, path_opt, "metadata", Some("pre-upload.json"));
    let post_upload =
        get_file_path(name, path_opt, "metadata", Some("post-upload.json"));

    (pre_upload, post_upload)
}

pub fn get_contract_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "contract", None)
}

pub fn get_toml_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "contract", Some("Move.toml"))
}

pub fn get_build_info_path(
    name: &str,
    path_opt: &Option<String>,
) -> Result<PathBuf> {
    // Note: This code block assumes that there is only one folder
    // in the build folder, which is the case.PathBuf::from(Path::new(project_dir.as_str()));
    let mut build_info_path =
        get_file_path(name, path_opt, "contract/build", None);
    let mut paths = fs::read_dir(&build_info_path).unwrap();

    if let Some(path) = paths.next() {
        build_info_path = path?.path();
        build_info_path.push("BuildInfo.yaml");
    } else {
        return Err(anyhow!("Could not find path to BuildInfo.yaml. Call `sui move build` to compile the Sui Move package"));
    }

    Ok(build_info_path)
}

fn get_file_path(
    name: &str,
    path_opt: &Option<String>,
    folder: &str,
    filename: Option<&str>,
) -> PathBuf {
    let mut filepath: PathBuf;

    if let Some(path) = path_opt {
        filepath = PathBuf::from(Path::new(path.clone().as_str()));
    } else {
        filepath = dirs::home_dir().unwrap();
        filepath.push(format!(".byte/projects/{}", name));
    }

    filepath.push(format!("{}/", folder));

    if let Some(file) = filename {
        filepath.push(file);
    }

    filepath
}

pub fn write_json(
    vec: Vec<String>,
    output_file: &Path,
) -> Result<(), anyhow::Error> {
    // Create the parent directories if they don't exist
    fs::create_dir_all(output_file.parent().unwrap())?;

    let file = File::create(output_file).map_err(|err| {
        anyhow!(
            r#"Could not create file "{}": {err}"#,
            output_file.display()
        )
    })?;

    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
    let ser = &mut serde_json::Serializer::with_formatter(file, formatter);
    vec.serialize(ser).map_err(|err| {
        anyhow!(r#"Could not write file "{}": {err}"#, output_file.display())
    })
}
