use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MoveToml {
    pub package: Package,
    pub dependencies: HashMap<String, Dependency>,
}

#[derive(Deserialize, Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub published_at: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Dependency {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}
