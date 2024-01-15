use anyhow::{anyhow, Result};
use convert_case::{Case, Casing};
use gutenberg_types::models::address::Address;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

use crate::version::Version;

/// Represents a Package Registry containing package information.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PackageRegistry(
    pub BTreeMap<String, BTreeMap<Version, PackageInfo>>,
);

impl PackageRegistry {
    /// Retrieves the object ID from the specified package name and revision.
    ///
    /// # Arguments
    ///
    /// * `pkg_name` - The name of the package.
    /// * `rev` - The revision to search for.
    ///
    /// # Returns
    ///
    /// * A reference to the object ID associated with the package and revision.
    pub fn get_object_id_from_rev(
        &self,
        pkg_name: String,
        rev: String,
    ) -> Result<&Address> {
        let versions = self.0.get(&pkg_name).ok_or_else(|| {
            anyhow!(format!(
                "Unable to find package '{}' in Registry",
                pkg_name
            ))
        })?;

        let (_, metadata) = versions
            .iter()
            .find(|(_v, metadata)| metadata.contract_ref.path.rev == rev)
            .unwrap();

        metadata.package.published_at.as_ref().ok_or_else(||
            anyhow!(format!(
                "Failed to retrieve `published-at` package address from package '{}'"
                , metadata.package.name)
            ))
    }

    /// Retrieves package information for the specified dependency and version.
    ///
    /// # Arguments
    ///
    /// * `dep_name` - The name of the dependency.
    /// * `version` - The version of the dependency.
    ///
    /// # Returns
    ///
    /// * A reference to the package information.
    pub fn get_package_info(
        &self,
        dep_name: &String,
        version: &Version,
    ) -> Result<&PackageInfo> {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).ok_or_else(|| {
            anyhow!(format!(
                "Could not find Package Name {} in thePackage Registry'",
                dep_name
            ))
        })?;

        let dependency = versions.get(version).ok_or_else(|| {
            anyhow!(format!("Unable to fetch {} v{}", dep_name, version))
        })?;

        Ok(dependency)
    }

    /// Retrieves packages with Git paths for the specified dependencies and
    /// version.
    ///
    /// # Arguments
    ///
    /// * `dep_names` - The names of the dependencies.
    /// * `version` - The version to fetch.
    ///
    /// # Returns
    ///
    /// * A map of dependency names to Git paths.
    pub fn get_packages_git(
        &self,
        dep_names: &[String],
        version: &Version,
    ) -> Result<BTreeMap<String, GitPath>> {
        dep_names
            .iter()
            .map(|dep_name| {
                Ok((
                    dep_name.clone(),
                    self.get_package_info(dep_name, version)?
                        .contract_ref
                        .path
                        .clone(),
                ))
            })
            .collect()
    }

    /// Retrieves a list of packages that require updating based on their
    /// dependencies.
    ///
    /// This function filters and maps the provided dependencies to their
    /// updated versions if available. It uses `get_updated_package_info` to
    /// check each dependency.
    ///
    /// # Arguments
    /// * `deps` - A slice of references to `PackageInfo` representing the
    ///   dependencies.
    ///
    /// # Returns
    /// A vector of references to `PackageInfo` containing the packages that
    /// need updating.
    pub fn get_packages_to_update(
        &self,
        deps: &[&PackageInfo],
    ) -> Vec<&PackageInfo> {
        deps.iter()
            .filter_map(|contract| self.get_updated_package_info(contract))
            .collect()
    }

    /// Determines if a given package dependency has a newer version available.
    ///
    /// This function checks if there is a newer version for the provided
    /// package. If a newer version exists, it returns a reference to the
    /// updated `PackageInfo`.
    ///
    /// # Arguments
    /// * `dep` - A reference to `PackageInfo` representing the package to
    ///   check.
    ///
    /// # Returns
    /// An `Option` containing a reference to updated `PackageInfo` if a newer
    /// version exists.
    pub fn get_updated_package_info(
        &self,
        dep: &PackageInfo,
    ) -> Option<&PackageInfo> {
        // Fetch available versions by package name
        let versions = if let Some(versions) = self.0.get(&dep.package.name) {
            versions
        } else {
            return None;
        };

        let latest_version = if let Some(latest) = versions.keys().max() {
            latest
        } else {
            return None;
        };

        // Safe to unwrap as `latest_version` was retrieved from
        // `versions.keys()`
        let latest = versions.get(latest_version).unwrap();

        (dep.package.version != latest.package.version).then_some(latest)
    }

    /// Fetches the latest version of a specified package.
    ///
    /// Attempts to retrieve the latest version available for a given package
    /// name. Returns an error if the package name is not found or if there
    /// is an issue retrieving the latest version.
    ///
    /// # Arguments
    /// * `dep_name` - The name of the dependency.
    ///
    /// # Returns
    /// A result containing the latest version or an error message.
    pub fn get_latest_version(&self, dep_name: &str) -> Result<&Version> {
        // Fetch available versions by package name
        let versions = self.0.get(dep_name).ok_or_else(|| {
            anyhow!(format!(
                "Could not find Package Name {} in PackageRegistry",
                dep_name
            ))
        })?;

        versions
            .keys()
            .max()
            // This error should not occur
            .ok_or_else(|| {
                anyhow!(format!(
                    "Unexpected error: Unable to retrieve latest version of {}",
                    dep_name
                ))
            })
    }

    /// Retrieves the version associated with a given object ID.
    ///
    /// This function iterates over all versions in the package map to find a
    /// match for the specified object ID. It returns an error if the object
    /// ID is not found.
    ///
    /// # Arguments
    /// * `object_id` - An `Address` reference representing the object ID.
    ///
    /// # Returns
    /// A result containing the matched version or an error message.
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
                    // This closure must return `bool` not wrapped by Result
                    // as this is the expected output to `find`
                    .expect("Error: PublishedAt field seems to be empty")
                    == object_id
            });

            if let Some(search_result) = search_result {
                return Ok(*search_result.0);
            }
        }

        Err(anyhow!("Unable to find object ID in package map"))
    }

    /// Retrieves the external dependency path for a given protocol and version.
    ///
    /// This function searches for a specific external dependency within a
    /// protocol's version and returns its Git path. An error is returned if
    /// the protocol or version is not found.
    ///
    /// # Arguments
    /// * `ext_dep` - The name of the external dependency.
    /// * `version` - The version of the protocol.
    ///
    /// # Returns
    /// A result containing the Git path of the external dependency or an error
    /// message.
    pub fn get_ext_dep_from_protocol(
        &self,
        ext_dep: &str,
        version: &Version,
    ) -> Result<GitPath> {
        // i.e. Sui or Originmate
        let protocol_versions =
            self.0.get(&String::from("NftProtocol")).ok_or_else(|| {
                anyhow!(format!(
                    "Could not find Package Name {} in PackageRegistry",
                    ext_dep
                ))
            })?;

        Ok(protocol_versions
            .get(version)
            .ok_or_else(|| {
                anyhow!(format!(
                    "Unable to fetch version '{}' for package 'NftProtocol'",
                    version
                ))
            })?
            .dependencies
            .get(ext_dep)
            .ok_or_else(|| {
                anyhow!(format!("Unable to fetch {} dependency", ext_dep))
            })?
            .path
            .clone())
    }
}

/// Represents package information, including dependencies and package
/// references.
///
/// This struct contains metadata about a package, including its dependencies
/// and the reference path to the package.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PackageInfo {
    pub package: Package,
    pub contract_ref: PackagePath,
    // TODO: Consider making this a self-similar struct, such that
    // we keep dependency tree's depth in its entirity
    pub dependencies: HashMap<String, PackagePath>,
}

/// Represents a path to a package, including the object ID and Git information.
///
/// This struct is used to store the path to a specific package and its
/// associated object ID.
#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct PackagePath {
    pub path: GitPath,
    pub object_id: Address,
}

/// Represents basic package metadata, including name, version, and publish
/// date.
///
/// This struct holds essential information about a package, such as its name,
/// version, and optional publish date.
#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct Package {
    name: String,
    version: Version,
    flavor: Flavor,
    #[serde(rename(serialize = "published-at"))]
    published_at: Option<Address>,
}

#[derive(Deserialize, Debug, Serialize, Clone, PartialEq, Eq)]
pub enum Flavor {
    Testnet,
    Mainnet,
}

impl Package {
    /// Constructs a new `Package` instance.
    ///
    /// Creates a new package with the specified name, version, and optional
    /// published address.
    ///
    /// # Arguments
    /// * `name` - The name of the package.
    /// * `version` - The version of the package.
    /// * `published_at` - Optional address indicating the published address.
    pub fn new(
        name: String,
        version: Version,
        flavor: Flavor,
        published_at: Option<Address>,
    ) -> Self {
        Self {
            name,
            version,
            flavor,
            published_at,
        }
    }

    /// Retrieves the package name, formatted in Pascal case.
    ///
    /// Converts and returns the package name to Pascal case format. Useful for
    /// consistent naming conventions in user-facing applications.
    ///
    /// # Returns
    /// The package name as a `String` in Pascal case.
    pub fn name(&self) -> String {
        self.name.as_str().to_case(Case::Pascal)
    }

    /// Retrieves a reference to the package version.
    ///
    /// Provides a convenient way to access the version of the package.
    ///
    /// # Returns
    /// A reference to the `Version` of the package.
    pub fn version(&self) -> &Version {
        &self.version
    }
}

/// Represents a Git repository path with optional subdirectory and revision
/// information.
///
/// This struct encapsulates the details required to specify a location within a
/// Git repository. It includes the main repository URL, an optional
/// subdirectory within that repository, and the revision (such as a commit
/// hash, branch, or tag) to pinpoint the exact state of the repository.
///
/// # Fields
/// * `git` - The URL of the Git repository. This should be a complete URL
///   pointing to a Git repo.
/// * `subdir` - An optional subdirectory within the Git repository. This allows
///   specifying a path within a larger repo, facilitating the use of mono-repos
///   or repositories with multiple projects.
/// * `rev` - The revision of the repository. This can be a commit hash, a
///   branch name, or a tag, depending on the needs of the application. It
///   specifies the exact state of the repository to be used.
#[derive(Deserialize, Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GitPath {
    pub git: String,
    pub subdir: Option<String>,
    pub rev: String,
}

impl GitPath {
    /// Constructs a new `GitPath`.
    ///
    /// Creates a `GitPath` instance representing the Git repository URL,
    /// an optional subdirectory within the repository, and the specific
    /// revision.
    ///
    /// # Arguments
    /// * `git` - The Git repository URL.
    /// * `subdir` - Optional subdirectory within the Git repository.
    /// * `rev` - The revision of the repository (commit hash, branch, or tag).
    pub fn new(git: String, subdir: Option<String>, rev: String) -> Self {
        Self { git, subdir, rev }
    }

    /// Sanitizes the `subdir` field by setting it to `None` if it's empty.
    ///
    /// Ensures the `subdir` field is meaningful by removing it if it's empty.
    /// This prevents potential issues with paths in Git operations.
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
    use super::*;
    use std::{env, fs::File, str::FromStr};

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
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let version_1 = Version::from_str(&String::from("1.0.0"))?;
        let version_1_2 = Version::from_str(&String::from("1.2.0"))?;

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
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let obj_v1 = registry.get_object_id_from_rev(
            String::from("NftProtocol"),
            V1_REV.to_string(),
        )?;

        assert_eq!(
            format!("{}", obj_v1),
            "0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9"
        );

        let obj_v1_2 = registry.get_object_id_from_rev(
            String::from("NftProtocol"),
            V1_2_REV.to_string(),
        )?;

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
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let obj_v1 = Address::new("0xbc3df36be17f27ac98e3c839b2589db8475fa07b20657b08e8891e3aaf5ee5f9")?;
        let v1 = registry.get_version_from_object_id(&obj_v1)?;
        assert_eq!(format!("{}", v1), "1.0.0");

        let obj_v2 = Address::new("0x77d0f09420a590ee59eeb5e39eb4f953330dbb97789e845b6e43ce64f16f812e")?;
        let v2 = registry.get_version_from_object_id(&obj_v2)?;
        assert_eq!(format!("{}", v2), "1.2.0");

        Ok(())
    }

    #[test]
    fn test_get_updated_package_info() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let old_package = PackageInfo {
            package: Package {
                name: String::from("Permissions"),
                version: Version::from_str("1.0.0")?,
                flavor: Flavor::Mainnet,
                published_at: Some(Address::new("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40")?),
            },
            contract_ref: PackagePath {
                path: GitPath {
                    git: String::from("https://github.com/Origin-Byte/nft-protocol.git"),
                    subdir: Some(String::from("contracts/permissions")),
                    rev: String::from("95d16538dc7688dd4c4a5e7c3348bf3addf9c310"),
                },
                object_id: Address::new("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40")?,
            },
            // No need to add dependencies as they're not used in this function
            dependencies: HashMap::new(),
        };

        let mut actual = registry
            .get_updated_package_info(&old_package)
            .unwrap()
            .clone();

        let expected = PackageInfo {
            package: Package {
                name: String::from("Permissions"),
                version: Version::from_str("1.2.0")?,
                flavor: Flavor::Mainnet,
                published_at: Some(Address::new("0xc8613b1c0807b0b9cfe229c071fdbdbc06a89cfe41e603c5389941346ad0b3c8")?),
            },
            contract_ref: PackagePath {
                path: GitPath {
                    git: String::from("https://github.com/Origin-Byte/nft-protocol.git"),
                    subdir: Some(String::from("contracts/permissions")),
                    rev: String::from("93f6cd0b8966354b1b00e7d798cbfddaa867a07b"),
                },
                object_id: Address::new("0x16c5f17f2d55584a6e6daa442ccf83b4530d10546a8e7dedda9ba324e012fc40")?,
            },
            // No need to add dependencies as they're not used in this function
            dependencies: HashMap::new(),
        };

        // Reset dependencies to match expected struct
        actual.dependencies = HashMap::new();

        assert_eq!(expected, actual);

        Ok(())
    }

    #[test]
    fn test_get_latest_version() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let actual = registry.get_latest_version("NftProtocol")?;
        let expected = Version::from_str("1.2.0")?;

        assert_eq!(actual, &expected);

        Ok(())
    }

    #[test]
    fn test_get_ext_dependency() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let actual = registry
            .get_ext_dep_from_protocol("Sui", &Version::from_str("1.0.0")?)?;

        assert_eq!(
            actual.git,
            String::from("https://github.com/MystenLabs/sui.git")
        );
        assert_eq!(
            actual.subdir,
            Some(String::from("crates/sui-framework/packages/sui-framework"))
        );
        assert_eq!(
            actual.rev,
            String::from("ae1212baf8f0837e25926d941db3d26a61c1bea2")
        );

        let actual = registry
            .get_ext_dep_from_protocol("Sui", &Version::from_str("1.2.0")?)?;

        assert_eq!(
            actual.git,
            String::from("https://github.com/MystenLabs/sui.git")
        );
        assert_eq!(
            actual.subdir,
            Some(String::from("crates/sui-framework/packages/sui-framework"))
        );
        assert_eq!(
            actual.rev,
            String::from("8b681515c0cf435df2a54198a28ab4ef574d202b")
        );

        let actual = registry.get_ext_dep_from_protocol(
            "Originmate",
            &Version::from_str("1.0.0")?,
        )?;

        assert_eq!(
            actual.git,
            String::from("https://github.com/Origin-Byte/originmate.git")
        );
        assert_eq!(actual.subdir, Some(String::from("")));
        assert_eq!(
            actual.rev,
            String::from("36e02283fa00451e8476a1bbc201af9a248396de")
        );

        let actual = registry.get_ext_dep_from_protocol(
            "Originmate",
            &Version::from_str("1.2.0")?,
        )?;

        assert_eq!(
            actual.git,
            String::from("https://github.com/Origin-Byte/originmate.git")
        );
        assert_eq!(actual.subdir, Some(String::from("")));
        assert_eq!(
            actual.rev,
            String::from("3e23d0707a346cf8780345611a2a25db3ec482b3")
        );

        Ok(())
    }

    #[test]
    fn test_get_pascal_name() -> Result<()> {
        let current_dir =
            env::current_dir().expect("Failed to retrieve current directory.");

        // Specify the file name or relative path of the file
        let file_path = "registry/registry-main.json";

        // Construct the full path to the file
        let current_dir = current_dir.join(file_path);

        let file = File::open(current_dir)?;
        let registry: PackageRegistry = serde_json::from_reader(file)?;

        let pkg = registry
            .0
            .get("NftProtocol")
            .unwrap()
            .get(&Version::from_str("1.0.0")?)
            .unwrap();

        assert_eq!(pkg.package.name(), "NftProtocol");

        let pkg = registry
            .0
            .get("LiquidityLayer")
            .unwrap()
            .get(&Version::from_str("1.0.0")?)
            .unwrap();

        assert_eq!(pkg.package.name(), "LiquidityLayer");

        Ok(())
    }
}
