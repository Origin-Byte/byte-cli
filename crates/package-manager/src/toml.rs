use anyhow::anyhow;
use console::style;
use gutenberg_types::models::address::Address;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

use crate::{
    package::{GitPath, Package, PackageInfo, PackagePath, PackageRegistry},
    version::Version,
};

/// Represents the structure of a Move.toml file, which includes package
/// metadata, dependencies, and addresses mapping.
///
/// This struct is used to parse and manipulate the contents of a Move.toml
/// file, commonly used in Move language projects for configuration and
/// dependency management.
///
/// # Fields
/// * `package` - Metadata about the package itself, including name, version,
///   and publish address.
/// * `dependencies` - A mapping of dependency names to their Git repository
///   paths.
/// * `addresses` - A mapping of names to blockchain addresses used in the Move
///   package.
#[derive(Deserialize, Debug, Serialize)]
pub struct MoveToml {
    package: Package,
    dependencies: BTreeMap<String, GitPath>,
    addresses: BTreeMap<String, Address>,
}

impl MoveToml {
    /// Constructs a new `MoveToml` instance.
    ///
    /// # Arguments
    /// * `package` - The metadata of the package.
    /// * `dependencies` - A map of dependency names to their Git paths.
    /// * `addresses` - A map of names to addresses.
    pub fn new(
        package: Package,
        dependencies: BTreeMap<String, GitPath>,
        addresses: BTreeMap<String, Address>,
    ) -> Self {
        Self {
            package,
            dependencies,
            addresses,
        }
    }

    /// Sanitizes dependency paths by removing empty subdirectories.
    ///
    /// Iterates through each dependency and cleans up its GitPath, removing
    /// any empty subdirectory fields.
    pub fn sanitize_output(&mut self) {
        self.dependencies
            .iter_mut()
            .for_each(|(_, dep)| dep.sanitize_subdir());
    }

    /// Retrieves a list of `PackageInfo` objects from the package registry.
    ///
    /// # Arguments
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    ///
    /// # Returns
    /// A result containing a reference to a map of `Version` to `PackageInfo`,
    /// or an error if the package is not found.
    pub fn pkg_info_list<'a>(
        &self,
        pkg_registry: &'a PackageRegistry,
    ) -> Result<&'a BTreeMap<Version, PackageInfo>, anyhow::Error> {
        let name = self.package.name();
        pkg_registry.0.get(&name).ok_or_else(|| {
            anyhow!("Could not find package '{name}' in Package Registry")
        })
    }

    /// Retrieves the `PackageInfo` for the current package from the registry.
    ///
    /// # Arguments
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    ///
    /// # Returns
    /// A result containing a reference to `PackageInfo`, or an error if not
    /// found.
    pub fn pkg_info<'a>(
        &self,
        pkg_registry: &'a PackageRegistry,
    ) -> Result<&'a PackageInfo, anyhow::Error> {
        let version = self.package.version();
        let info_list = self.pkg_info_list(pkg_registry)?;

        info_list.get(&version).ok_or_else(|| {
            anyhow!(
                "Could not find version '{version}' for Package '{}'",
                self.package.name()
            )
        })
    }

    /// Retrieves a vector of `PackageInfo` for each dependency in the package.
    ///
    /// # Arguments
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    ///
    /// # Returns
    /// A vector of references to `PackageInfo` for each dependency.
    pub fn dependency_pkg_infos<'a>(
        &'a self,
        pkg_registry: &'a PackageRegistry,
    ) -> Vec<&'a PackageInfo> {
        self.dependencies
            .iter()
            .filter_map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name);

                if let Some(pack) = dep_pack {
                    Some(get_package_info(specs, pack))
                } else {
                    println!("{} Skipping Package {:?}, could not find it in the Package Registry", style("Warning ").yellow().bold(), name);
                    None
                }
            })
            .collect()
    }

    /// Retrieves a list of addresses corresponding to the dependencies in the
    /// `MoveToml`.
    ///
    /// This function maps each dependency in the `MoveToml` to its associated
    /// address from the package registry. It is used to fetch the addresses
    /// of all dependencies listed in the Move.toml file.
    ///
    /// # Arguments
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    ///
    /// # Returns
    /// A vector of references to `Address` for each dependency.
    pub fn get_dependency_ids<'a>(
        &'a self,
        pkg_registry: &'a PackageRegistry,
    ) -> Vec<&'a Address> {
        self.dependencies
            .iter()
            .map(|(name, specs)| {
                let dep_pack = pkg_registry.0.get(name).expect(
                    format!(
                        "Could not find Package Name {} in PackageRegistry",
                        name
                    )
                    .as_str(),
                );

                get_object_id_from_rev(dep_pack, &specs.rev)
            })
            .collect()
    }

    /// Updates the `MoveToml` file with the latest package versions and
    /// dependencies.
    ///
    /// This function checks for updates in the package registry and updates the
    /// Move.toml file accordingly. It also sanitizes the paths of updated
    /// dependencies.
    ///
    /// # Arguments
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    pub fn update_toml(&mut self, pkg_registry: &PackageRegistry) {
        let dependencies = self.dependency_pkg_infos(pkg_registry);

        let to_update = pkg_registry.get_packages_to_update(&dependencies);

        let mut updated_deps = to_update
            .iter()
            .map(|dep| {
                (
                    dep.package.name(),
                    dep.contract_ref.path.clone(),
                    dep.package.version().clone(),
                )
            })
            .collect::<Vec<(String, GitPath, Version)>>();

        updated_deps
            .drain(..)
            .for_each(|(dep_name, mut dep, dep_version)| {
                println!(
                    "{}{}",
                    style("Updated ").green().bold().on_bright(),
                    format!("{} to version {}", dep_name, dep_version).as_str()
                );

                dep.sanitize_subdir();

                self.dependencies.insert(dep_name, dep);
            });
    }

    /// Generates a `MoveToml` instance with specified dependencies and version.
    ///
    /// This function creates a `MoveToml` instance with a specific set of
    /// dependencies, external dependencies, and a version. It is
    /// particularly useful for initializing a Move.toml file with
    /// predefined settings.
    ///
    /// # Arguments
    /// * `name` - The name of the package.
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    /// * `dep_names` - A list of dependency names.
    /// * `ext_dep_names` - A list of external dependency names.
    /// * `version` - The specific version of the package.
    ///
    /// # Returns
    /// A result containing the new `MoveToml` instance or an error.
    pub fn get_toml(
        name: &str,
        pkg_registry: &PackageRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
        version: &Version,
    ) -> Result<Self, anyhow::Error> {
        let empty_addr = Address::new("0x0")?;

        let mut dependencies =
            pkg_registry.get_packages_git(dep_names, version)?;

        // Inserts Sui and Originmate
        ext_dep_names
            .iter()
            .try_for_each::<_, Result<(), anyhow::Error>>(|dep_name| {
                let ext_dep = pkg_registry
                    .get_ext_dep_from_protocol(dep_name.as_str(), version)?;

                dependencies.insert(dep_name.clone(), ext_dep);

                Ok(())
            })?;

        let toml = MoveToml {
            package: Package::new(
                name.to_string(),
                Version::from_str("1.0.0")?,
                Some(empty_addr.clone()),
            ),
            dependencies,
            addresses: BTreeMap::from([(String::from(name), empty_addr)]),
        };

        Ok(toml)
    }

    /// Generates the latest `MoveToml` instance for a package.
    ///
    /// This function is similar to `get_toml` but automatically uses the latest
    /// version from the package registry. It is used when initializing a
    /// Move.toml file with the most recent versions of dependencies.
    ///
    /// # Arguments
    /// * `name` - The name of the package.
    /// * `pkg_registry` - A reference to the `PackageRegistry`.
    /// * `dep_names` - A list of dependency names.
    /// * `ext_dep_names` - A list of external dependency names.
    ///
    /// # Returns
    /// A result containing the new `MoveToml` instance or an error.
    pub fn get_toml_latest(
        name: &str,
        pkg_registry: &PackageRegistry,
        dep_names: &[String],
        ext_dep_names: &[String],
    ) -> Result<Self, anyhow::Error> {
        // Oath of honor --> Monolitic release (for now)
        let version =
            pkg_registry.get_latest_version(&String::from("NftProtocol"))?;

        MoveToml::get_toml(
            name,
            pkg_registry,
            dep_names,
            ext_dep_names,
            version,
        )
    }

    /// Retrieves a specific dependency's `GitPath` from the `MoveToml`.
    ///
    /// This function fetches the GitPath for a named dependency from the
    /// Move.toml file.
    ///
    /// # Arguments
    /// * `dep_name` - The name of the dependency.
    ///
    /// # Returns
    /// A reference to the `GitPath` of the specified dependency.
    pub fn get_dependency<'a>(&'a self, dep_name: &'a str) -> &'a GitPath {
        // Fetch available versions by package name
        let dependency = self.dependencies.get(dep_name).expect(
            format!("Could not find GitPath Name {} in Move.toml", dep_name)
                .as_str(),
        );

        dependency
    }

    /// Converts the `MoveToml` instance into a TOML value.
    ///
    /// This function serializes the `MoveToml` instance into a TOML value,
    /// suitable for saving to a file or further manipulation.
    ///
    /// # Returns
    /// A result containing the TOML value or a serialization error.
    pub fn to_value(&self) -> Result<toml::Value, toml::ser::Error> {
        toml::Value::try_from(self)
    }

    /// Converts the `MoveToml` instance into a formatted TOML string.
    ///
    /// This function serializes the `MoveToml` instance into a prettified TOML
    /// string, adding appropriate vertical spacing for better readability.
    ///
    /// # Returns
    /// A result containing the formatted TOML string or a serialization error.
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        let mut toml_string = toml::to_string_pretty(self)?;
        toml_string = add_vertical_spacing(toml_string.as_str());

        Ok(toml_string)
    }
}

/// Retrieves `PackageInfo` for a given dependency based on its GitPath.
///
/// This function is used to fetch the package information associated with a
/// specific revision of a dependency, as identified by its `GitPath`.
///
/// # Arguments
/// * `dependency` - A reference to the `GitPath` of the dependency.
/// * `versions` - A reference to a `BTreeMap` mapping `Version` to
///   `PackageInfo`.
///
/// # Returns
/// A reference to the `PackageInfo` corresponding to the specified dependency
/// revision.
pub fn get_package_info<'a>(
    dependency: &'a GitPath,
    versions: &'a BTreeMap<Version, PackageInfo>,
) -> &'a PackageInfo {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    contract
}

/// Retrieves the version and corresponding package information for a specific
/// revision.
///
/// This function finds the version and `PackageInfo` in the provided versions
/// map that matches the specified revision.
///
/// # Arguments
/// * `versions` - A reference to a `BTreeMap` mapping `Version` to
///   `PackageInfo`.
/// * `rev` - A reference to a `String` representing the revision.
///
/// # Returns
/// A tuple containing a reference to the `Version` and a reference to the
/// `PackageInfo`.
pub fn get_version_and_pkg_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> (&'a Version, &'a PackageInfo) {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
}

/// Retrieves `PackageInfo` for a given revision.
///
/// This function looks up the `PackageInfo` for a specific revision within the
/// versions map.
///
/// # Arguments
/// * `versions` - A reference to a `BTreeMap` mapping `Version` to
///   `PackageInfo`.
/// * `rev` - A reference to a `String` representing the revision.
///
/// # Returns
/// A reference to the `PackageInfo` for the specified revision.
pub fn get_package_info_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> &'a PackageInfo {
    versions
        .iter()
        .find(|(_, contract)| contract.contract_ref.path.rev == *rev)
        .expect(format!("Could not find rev {} in version map", rev).as_str())
        .1
}

/// Retrieves the object ID for a given revision from the versions map.
///
/// This function finds the object ID associated with a specific revision of a
/// package. The object ID is part of the `PackageInfo` and is used for
/// identifying the package.
///
/// # Arguments
/// * `versions` - A reference to a `BTreeMap` mapping `Version` to
///   `PackageInfo`.
/// * `rev` - A reference to a `String` representing the revision.
///
/// # Returns
/// A reference to the `Address` object ID for the specified revision.
pub fn get_object_id_from_rev<'a>(
    versions: &'a BTreeMap<Version, PackageInfo>,
    rev: &'a String,
) -> &'a Address {
    println!("Getting object ID from ");
    let contract = get_package_info_from_rev(versions, rev);

    &contract.contract_ref.object_id
}

/// Generates a `PackagePath` for a given dependency.
///
/// This function constructs a `PackagePath`, including both the path and object
/// ID, for a specific dependency based on its `GitPath` and the versions map.
///
/// # Arguments
/// * `dependency` - A reference to the `GitPath` of the dependency.
/// * `versions` - A reference to a `BTreeMap` mapping `Version` to
///   `PackageInfo`.
///
/// # Returns
/// A `PackagePath` consisting of the dependency's path and its associated
/// object ID.
pub fn get_contract_ref(
    dependency: &GitPath,
    versions: &BTreeMap<Version, PackageInfo>,
) -> PackagePath {
    let (_, contract) =
        get_version_and_pkg_info_from_rev(versions, &dependency.rev);

    PackagePath {
        path: dependency.clone(),
        object_id: contract.contract_ref.object_id.clone(),
    }
}

/// This function is here because Toml serializer seems to be
/// failing to add a vertical space between the tables `package` and
/// `dependencies`
///
/// Adds vertical spacing to a TOML string.
///
/// This function is a workaround for a known issue where the TOML serializer
/// does not add vertical space between tables. It ensures better readability
/// of the serialized TOML string.
///
/// # Arguments
/// * `input` - The original TOML string.
///
/// # Returns
/// The modified TOML string with added vertical spacing.
pub fn add_vertical_spacing(input: &str) -> String {
    let re = Regex::new(r"(?m)^(published-at.*)")
        .expect("Failed to read `published-at` field");
    re.replace_all(input, "$1\n").to_string()
}
