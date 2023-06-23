use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use gutenberg::models::Address;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::version::Version;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgRegistry(pub BTreeMap<String, BTreeMap<Version, PkgInfo>>);

impl PkgRegistry {
    pub fn get_object_id_from_rev(
        &self,
        pkg_name: String,
        rev: String,
    ) -> &Address {
        let versions = &self.0.get(&pkg_name).unwrap();

        let (_, metadata) = versions
            .iter()
            .find(|(_v, metadata)| metadata.contract_ref.path.rev == rev)
            .unwrap();

        metadata.package.published_at.as_ref().unwrap()
    }

    pub fn get_pkg_info<'a>(
        &'a self,
        dep_name: &'a String,
        version: &'a Version,
    ) -> &'a PkgInfo {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).expect(
            format!("Could not find Package Name {} in PkgRegistry", dep_name)
                .as_str(),
        );

        let dependency = versions.get(version).expect(
            format!("Unable to fetch {} v{}", dep_name, version).as_str(),
        );

        dependency
    }

    pub fn get_pkgs_git(
        &self,
        dep_names: &Vec<String>,
        version: &Version,
    ) -> HashMap<String, GitPath> {
        let deps = dep_names
            .iter()
            .map(|dep_name| {
                (
                    dep_name.clone(),
                    self.get_pkg_info(dep_name, version)
                        .contract_ref
                        .path
                        .clone(),
                )
            })
            .collect::<HashMap<String, GitPath>>();

        deps
    }

    pub fn get_pkgs_to_update<'a>(
        &'a self,
        deps: &'a [&'a PkgInfo],
    ) -> Vec<&'a PkgInfo> {
        let mut to_update: Vec<&'a PkgInfo> = vec![];

        deps.iter().for_each(|contract| {
            if let Some(update) = self.get_updated_pkg_info(contract) {
                to_update.push(update);
            }
        });

        to_update
    }

    pub fn get_updated_pkg_info<'a>(
        &'a self,
        dep: &'a PkgInfo,
    ) -> Option<&'a PkgInfo> {
        // Fetch available versions by package name
        let versions = self.0.get(&dep.package.name).expect(
            format!(
                "Could not find Package Name {} in PkgRegistry",
                &dep.package.name
            )
            .as_str(),
        );

        let latest_version = versions
            .keys()
            .max()
            // This error should not occur
            .expect(
                format!(
                    "Unexpected error: Unable to retrieve latest version of {}",
                    &dep.package.name
                )
                .as_str(),
            );

        let latest = versions.get(latest_version).unwrap();

        (dep.package.version != latest.package.version).then_some(latest)
    }

    pub fn get_latest_protocol_version<'a>(
        &'a self,
        dep_name: &String,
    ) -> &'a Version {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).expect(
            format!("Could not find Package Name {} in PkgRegistry", dep_name)
                .as_str(),
        );

        versions
            .keys()
            .max()
            // This error should not occur
            .expect(
                format!(
                    "Unexpected error: Unable to retrieve latest version of {}",
                    dep_name
                )
                .as_str(),
            )
    }

    pub fn get_version_from_object_id(
        &self,
        object_id: &Address,
    ) -> Result<Version> {
        for (_, version_map) in self.0.iter() {
            let search_result = version_map.iter().find(|(_, contract)| {
                contract
                    .package
                    .published_at
                    .as_ref()
                    .expect("Error: PublishedAt field seems to be empty")
                    == object_id
            });

            if let Some(search_result) = search_result {
                return Ok(*search_result.0);
            }
        }

        Err(anyhow!("Unable to find object ID in package map"))
    }

    // i.e. Sui or Originmate
    pub fn get_ext_dep_from_protocol(
        &self,
        ext_dep: String,
        version: &Version,
    ) -> GitPath {
        let protocol_versions =
            self.0.get(&String::from("NftProtocol")).expect(
                format!(
                    "Could not find Package Name {} in PkgRegistry",
                    &ext_dep
                )
                .as_str(),
            );

        protocol_versions
            .get(version)
            .expect(
                format!(
                    "Unable to fetch version '{}' for package 'NftProtocol'",
                    version
                )
                .as_str(),
            )
            .dependencies
            .get(&ext_dep)
            .expect(format!("Unable to fetch {} dependency", ext_dep).as_str())
            .path
            .clone()
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PkgInfo {
    pub package: Package,
    pub contract_ref: PkgPath,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, PkgPath>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PkgPath {
    pub path: GitPath,
    pub object_id: Address,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Package {
    pub name: String,
    pub version: Version,
    #[serde(rename(serialize = "published-at"))]
    pub published_at: Option<Address>,
}

impl Package {
    pub fn name_pascal(&self) -> String {
        self.name.as_str().to_case(Case::Pascal)
    }
}

#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GitPath {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}

impl GitPath {
    pub fn sanitize_subdir(&mut self) {
        if let Some(inner) = &mut self.subdir {
            if inner.is_empty() {
                self.subdir = None;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{pkg::PkgRegistry, version::Version};

    use anyhow::Result;
    use gutenberg::models::Address;
    use std::{collections::HashMap, env, fs::File};

    use super::{GitPath, Package, PkgInfo, PkgPath};

    const V1_REV: &str = "95d16538dc7688dd4c4a5e7c3348bf3addf9c310";
    const V1_2_REV: &str = "93f6cd0b8966354b1b00e7d798cbfddaa867a07b";

    #[test]
    fn serialize_pkg_registry() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PkgRegistry = serde_json::from_reader(file)?;

        let version_1 = Version::from_string(&String::from("1.0.0"))?;
        let version_1_2 = Version::from_string(&String::from("1.2.0"))?;

        let nft_protocol_versions = registry.0.get("NftProtocol");

        let nft_protocol_version_1 =
            nft_protocol_versions.unwrap().get(&version_1).unwrap();

        assert_eq!(nft_protocol_version_1.contract_ref.path.rev, V1_REV);

        assert_eq!(
            format!("{}", nft_protocol_version_1.contract_ref.object_id),
            "0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9"
        );

        let nft_protocol_version_1_2 =
            nft_protocol_versions.unwrap().get(&version_1_2).unwrap();

        assert_eq!(nft_protocol_version_1_2.contract_ref.path.rev, V1_2_REV);

        // ObjectID remains the same, since it refers to the original package
        assert_eq!(
            format!("{}", nft_protocol_version_1_2.contract_ref.object_id),
            "0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9"
        );

        // PublishedAt points to the new object published for this version
        assert_eq!(
            format!("{}", nft_protocol_version_1_2.package.published_at.as_ref().unwrap()),
            "0x77d0f09420a590ee59eeb5e39eb4f953330dbb97789e845b6e43ce64f16f812e"
        );

        Ok(())
    }

    #[test]
    fn test_get_obj_id_from_rev() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PkgRegistry = serde_json::from_reader(file)?;

        let obj_v1 = registry.get_object_id_from_rev(
            String::from("NftProtocol"),
            V1_REV.to_string(),
        );

        assert_eq!(
            format!("{}", obj_v1),
            "0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9"
        );

        let obj_v1_2 = registry.get_object_id_from_rev(
            String::from("NftProtocol"),
            V1_2_REV.to_string(),
        );

        assert_eq!(
            format!("{}", obj_v1_2),
            "0x77d0f09420a590ee59eeb5e39eb4f953330dbb97789e845b6e43ce64f16f812e"
        );

        Ok(())
    }

    #[test]
    fn test_get_version_from_obj_id() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PkgRegistry = serde_json::from_reader(file)?;

        let obj_v1 = Address::new(String::from("0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9"))?;
        let v1 = registry.get_version_from_object_id(&obj_v1)?;
        assert_eq!(format!("{}", v1), "1.0.0");

        let obj_v2 = Address::new(String::from("0x77d0f09420a590ee59eeb5e39eb4f953330dbb97789e845b6e43ce64f16f812e"))?;
        let v2 = registry.get_version_from_object_id(&obj_v2)?;
        assert_eq!(format!("{}", v2), "1.2.0");

        Ok(())
    }

    #[test]
    fn test_get_updated_pkg_info() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PkgRegistry = serde_json::from_reader(file)?;

        let old_package = PkgInfo {
            package: Package {
                name: String::from("Permissions"),
                version: Version::from_string("1.0.0")?,
                published_at: Some(Address::new(String::from("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40"))?),
            },
            contract_ref: PkgPath {
                path: GitPath {
                    git: String::from("https://github.com/Origin-Byte/nft-protocol.git"),
                    subdir: Some(String::from("contracts/permissions")),
                    rev: String::from("95d16538dc7688dd4c4a5e7c3348bf3addf9c310"),
                },
                object_id: Address::new(String::from("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40"))?,
            },
            // No need to add dependencies as they're not used in this function
            dependencies: HashMap::new(),
        };

        let mut actual =
            registry.get_updated_pkg_info(&old_package).unwrap().clone();

        let expected = PkgInfo {
            package: Package {
                name: String::from("Permissions"),
                version: Version::from_string("1.2.0")?,
                published_at: Some(Address::new(String::from("0xc8613b1c0807b0b9cfe229c071fdbdbc06a89cfe41e603c5389941346ad0b3c8"))?),
            },
            contract_ref: PkgPath {
                path: GitPath {
                    git: String::from("https://github.com/Origin-Byte/nft-protocol.git"),
                    subdir: Some(String::from("contracts/permissions")),
                    rev: String::from("93f6cd0b8966354b1b00e7d798cbfddaa867a07b"),
                },
                object_id: Address::new(String::from("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40"))?,
            },
            // No need to add dependencies as they're not used in this function
            dependencies: HashMap::new(),
        };

        // Reset dependencies to match expected struct
        actual.dependencies = HashMap::new();

        assert_eq!(expected, actual);

        Ok(())
    }

    // TODO
    // get latest dependency
    // get_ext_dep_from_protocol
    // name_pascal
}
