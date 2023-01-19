use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Move {
    pub package: Package,
    pub dependencies: Dependencies,
    pub addresses: Addresses,
}

#[derive(Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize)]
pub struct Dependency {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}

impl Dependency {
    pub fn new(git: String, rev: String) -> Self {
        Self {
            git,
            subdir: None,
            rev: rev.trim_start_matches("0x").to_string(),
        }
    }

    pub fn subdir(mut self, subdir: String) -> Self {
        self.subdir = Some(subdir);
        self
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Addresses(Map<String>);

impl Addresses {
    pub fn new<T>(addresses: T) -> Self
    where
        T: Into<HashMap<String, String>>,
    {
        Self(Map::new(addresses.into()))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Dependencies(Map<Dependency>);

impl Dependencies {
    pub fn new<T>(dependencies: T) -> Self
    where
        T: Into<HashMap<String, Dependency>>,
    {
        Self(Map::new(dependencies.into()))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct Map<T>(HashMap<String, T>);

impl<T> Map<T> {
    pub fn new(map: HashMap<String, T>) -> Self {
        Self(map)
    }
}
