use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::err::GutenError;

#[derive(Debug, Default, Deserialize, Serialize, Reflect)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    #[serde(default)]
    pub display: bool,
    #[serde(default)]
    pub url: bool,
    #[serde(default)]
    pub attributes: bool,
    #[serde(default)]
    pub tags: bool,
}

pub enum MintType {
    Direct,
    Airdrop,
    Launchpad,
}

impl NftData {
    pub fn new(fields_vec: Vec<String>) -> Result<NftData, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let fields = NftData::fields();

        let field_struct = fields
            .iter()
            .map(|f| {
                let v = fields_to_add.contains(f);
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        NftData::from_map(&field_struct)
    }

    pub fn is_empty(&self) -> bool {
        !self.display && !self.url && !self.attributes && !self.tags
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<NftData, GutenError> {
        let mut field_struct = NftData::default();

        for (f, v) in map {
            match f.as_str() {
                "display" => {
                    field_struct.display = *v;
                    Ok(())
                }
                "url" => {
                    field_struct.url = *v;
                    Ok(())
                }
                "attributes" => {
                    field_struct.attributes = *v;
                    Ok(())
                }
                "tags" => {
                    field_struct.tags = *v;
                    Ok(())
                }
                other => Err(GutenError::UnsupportedNftInput(format!(
                    "The NFT field `{}` provided is not a supported",
                    other
                ))),
            }?;
        }

        Ok(field_struct)
    }

    pub fn fields() -> Vec<String> {
        let field_struct = NftData::default();
        let mut fields: Vec<String> = Vec::new();

        for (i, _) in field_struct.iter_fields().enumerate() {
            let field_name = field_struct.name_at(i).unwrap();

            fields.push(field_name.to_string());
        }
        fields
    }

    pub fn to_map(&self) -> Vec<(String, bool)> {
        let mut map: Vec<(String, bool)> = Vec::new();

        for (i, value) in self.iter_fields().enumerate() {
            let field_name = self.name_at(i).unwrap();
            let value_ = value.downcast_ref::<bool>().unwrap();
            map.push((field_name.to_string(), *value_));
        }
        map
    }

    pub fn has_display_domains(&self) -> bool {
        self.display || self.url || self.attributes
    }
}
