use std::path::{Path, PathBuf};

pub fn get_contract_path(name: &str, path_opt: &Option<String>) -> PathBuf {
    get_file_path(name, path_opt, "contract", None)
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
