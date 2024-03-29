use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};

use anyhow::Result;
use dashmap::DashMap;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use sui_types::transaction::CallArg;
use url::Url;

/// Represents a collection of metadata, using a concurrent hash map.
#[derive(Debug)]
pub struct GlobalMetadata(pub DashMap<u32, Metadata>);

/// Represents metadata that can be stored, using a binary tree map.
#[derive(Debug, Deserialize, Serialize)]
pub struct StorableMetadata(pub BTreeMap<u32, Metadata>);

/// Defines the metadata associated with an object.
#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub url: Option<Url>,
    pub description: Option<String>,
    pub attributes: Option<Vec<Trait>>,
}

/// Represents a single trait in the metadata.
#[derive(Debug, Deserialize, Serialize)]
pub struct Trait {
    trait_type: String,
    value: String,
}

impl GlobalMetadata {
    /// Converts `GlobalMetadata` into a `BTreeMap`.
    ///
    /// # Returns
    /// `BTreeMap<u32, Metadata>` - A binary tree map of the metadata.
    pub fn into_map(self) -> BTreeMap<u32, Metadata> {
        let GlobalMetadata(dash_map) = self;
        let mut hash_map = BTreeMap::new();

        dash_map.into_iter().for_each(|(k, e)| {
            hash_map.insert(k, e);
        });

        hash_map
    }

    /// Creates a `GlobalMetadata` from a `StorableMetadata`.
    ///
    /// # Arguments
    /// * `map` - `StorableMetadata` to convert from.
    ///
    /// # Returns
    /// `GlobalMetadata` instance.
    pub fn from_map(map: StorableMetadata) -> Self {
        GlobalMetadata(map.0.into_iter().collect())
    }
}

impl StorableMetadata {
    /// Creates a `StorableMetadata` from a `BTreeMap`.
    ///
    /// # Arguments
    /// * `hash_map` - `BTreeMap` to convert from.
    ///
    /// # Returns
    /// `StorableMetadata` instance.
    pub fn from_map(hash_map: BTreeMap<u32, Metadata>) -> Self {
        Self(hash_map)
    }

    /// Retrieves a set of indices for metadata entries that need to be
    /// uploaded.
    ///
    /// # Returns
    /// `BTreeSet<u32>` - A set of indices.
    pub fn get_to_upload(&self) -> BTreeSet<u32> {
        self.0
            .iter()
            .filter_map(|(idx, meta)| match meta.url {
                Some(_) => None,
                None => Some(*idx),
            })
            .collect()
    }
}

impl Metadata {
    /// Converts `Metadata` into a vector of `CallArg` for transaction purposes.
    ///
    /// # Returns
    /// `Result<Vec<CallArg>>` - A result containing a vector of `CallArg`.
    pub fn into_args(self) -> Result<Vec<CallArg>> {
        let mut params: Vec<CallArg> = Vec::new();

        if let Some(value) = &self.name {
            params.push(CallArg::Pure(bcs::to_bytes(value).unwrap()));
        }

        if let Some(value) = &self.description {
            params.push(CallArg::Pure(bcs::to_bytes(value).unwrap()));
        }

        if let Some(value) = &self.url {
            params.push(CallArg::Pure(bcs::to_bytes(value).unwrap()));
        }

        if let Some(map) = &self.attributes {
            let (keys, values): (Vec<String>, Vec<String>) = map
                .iter()
                .map(|att| (att.trait_type.clone(), att.value.clone()))
                .unzip();

            params.push(CallArg::Pure(bcs::to_bytes(&keys).unwrap()));
            params.push(CallArg::Pure(bcs::to_bytes(&values).unwrap()));
        }

        Ok(params)
    }
}

/// Visitor for deserializing `GlobalMetadata`.
struct GlobalMetadataVisitor {
    marker: PhantomData<fn() -> GlobalMetadata>,
}

impl GlobalMetadataVisitor {
    /// Creates a new `GlobalMetadataVisitor`.
    ///
    /// # Returns
    /// A new instance of `GlobalMetadataVisitor`.
    fn new() -> Self {
        GlobalMetadataVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for GlobalMetadataVisitor {
    type Value = GlobalMetadata;

    /// Specifies the expected input format for deserialization.
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "Unable to deserialize DashMap")
    }

    /// Deserializes a map into `GlobalMetadata`.
    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let dash_map = DashMap::with_capacity(access.size_hint().unwrap_or(0));

        // While there are entries remaining in the input, add them
        // into our map.
        while let Some((key, value)) = access.next_entry()? {
            dash_map.insert(key, value);
        }

        Ok(GlobalMetadata(dash_map))
    }
}

// This is the trait that informs Serde how to deserialize Version.
impl<'de> Deserialize<'de> for GlobalMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Instantiate VersionVisitor and ask the Deserializer to drive
        // it over the input data, resulting in an instance of Version.
        deserializer.deserialize_map(GlobalMetadataVisitor::new())
    }
}
