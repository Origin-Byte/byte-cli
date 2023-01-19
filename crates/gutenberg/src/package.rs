use serde::{Deserialize, Serialize};
use toml::value::Table;

#[derive(Serialize, Deserialize)]
pub struct Move {
    pub package: Package,
    pub dependencies: Dependencies,
    pub addresses: Table,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Dependencies {
    pub sui: Dependency,
    pub originmate: Dependency,
    pub nft_protocol: Dependency,
}

#[derive(Serialize, Deserialize)]
pub struct Dependency {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}
