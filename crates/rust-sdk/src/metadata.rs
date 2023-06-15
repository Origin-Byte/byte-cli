use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
    str::FromStr,
};

use anyhow::Result;
use dashmap::DashMap;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::json;
use sui_sdk::json::SuiJsonValue;
use url::Url;

#[derive(Debug)]
pub struct GlobalMetadata(pub DashMap<u32, Metadata>);

#[derive(Debug, Deserialize, Serialize)]
pub struct StorableMetadata(pub BTreeMap<u32, Metadata>);

#[derive(Debug, Deserialize, Serialize)]
pub struct Metadata {
    pub name: Option<String>,
    pub url: Option<Url>,
    pub description: Option<String>,
    pub attributes: Option<Vec<Trait>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Trait {
    trait_type: String,
    value: String,
}

impl GlobalMetadata {
    pub fn into_map(self) -> BTreeMap<u32, Metadata> {
        let GlobalMetadata(dash_map) = self;
        let mut hash_map = BTreeMap::new();

        dash_map.into_iter().for_each(|(k, e)| {
            hash_map.insert(k, e);
        });

        hash_map
    }

    pub fn from_map(map: StorableMetadata) -> Self {
        GlobalMetadata(map.0.into_iter().collect())
    }
}

impl StorableMetadata {
    pub fn from_map(hash_map: BTreeMap<u32, Metadata>) -> Self {
        Self(hash_map)
    }

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
    pub fn to_map(&self) -> Result<Vec<SuiJsonValue>> {
        let mut params: Vec<SuiJsonValue> = Vec::new();

        if let Some(value) = &self.name {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.url {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(value) = &self.description {
            params.push(SuiJsonValue::from_str(value.as_str())?);
        }

        if let Some(map) = &self.attributes {
            let (keys, values): (Vec<String>, Vec<String>) = map
                .iter()
                .map(|att| (att.trait_type.clone(), att.value.clone()))
                .unzip();

            let keys_arr = json!(keys);
            let values_arr = json!(values);

            params.push(SuiJsonValue::new(keys_arr)?);
            params.push(SuiJsonValue::new(values_arr)?);
        }

        Ok(params)
    }
}

struct GlobalMetadataVisitor {
    marker: PhantomData<fn() -> GlobalMetadata>,
}

impl GlobalMetadataVisitor {
    fn new() -> Self {
        GlobalMetadataVisitor {
            marker: PhantomData,
        }
    }
}

impl<'de> Visitor<'de> for GlobalMetadataVisitor {
    type Value = GlobalMetadata;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> fmt::Result {
        write!(formatter, "Unable to deserialize DashMap")
    }

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
