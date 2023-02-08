use std::collections::HashSet;

use crate::{err::GutenError, models::tags::Tags};

use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};

use super::{nft::NftData, royalties::Royalties};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub tags: Option<Tags>,   // Done
    pub royalties: Royalties, // Done
    pub mint_strategy: MintStrategy,
    pub composability: Option<Composability>,
    pub loose: bool,
    pub supply_policy: SupplyPolicy,
}

// impl Default for Settings {
//     fn default() -> Self {
//         Settings {
//             tags: None,
//             royalties: Royalties::default(),
//             mint_strategy: MintStrategy::default(),
//             composability: None,
//             loose: false,
//             supply_policy: SupplyPolicy::default(),
//         }
//     }
// }

impl Settings {
    pub fn new(
        tags: Option<Tags>,
        royalties: Royalties,
        mint_strategy: MintStrategy,
        composability: Option<Composability>,
        loose: bool,
        supply_policy: SupplyPolicy,
    ) -> Settings {
        Settings {
            tags,
            royalties,
            mint_strategy,
            composability,
            loose,
            supply_policy,
        }
    }

    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = Option::Some(tags);
    }

    pub fn set_royalties(&mut self, royalties: Royalties) {
        self.royalties = royalties;
    }

    pub fn write_tags(&self) -> String {
        self.tags
            .expect("No collection tags setup found")
            .write_domain(true)
    }

    pub fn write_royalties(&self) -> String {
        self.royalties.write()
    }

    pub fn write_type_declarations(&self) -> String {
        match self.composability {
            Some(composability) => composability.write_types(),
            None => "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Composability {
    types: Vec<String>,
    blueprint: Vec<(String, Link)>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Link {
    pub order: u64,
    pub limit: u64,
}

impl Composability {
    pub fn write_types(&self) -> String {
        let mut types = String::new();

        self.types
            .iter()
            .map(|t| {
                types.push_str(&format!(
                    "struct {} has copy, drop, store {{}} \n",
                    t
                ));
            })
            .collect::<()>();

        types
    }
}

pub enum MintType {
    Direct,
    Airdrop,
    Launchpad,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplyPolicy {
    Unlimited,
    Limited { max: u64 },
    Undefined,
}

impl Default for SupplyPolicy {
    fn default() -> Self {
        SupplyPolicy::Undefined
    }
}

impl SupplyPolicy {
    pub fn new(
        input: &str,
        supply: Option<u64>,
    ) -> Result<SupplyPolicy, GutenError> {
        match input {
            "Unlimited" => Ok(SupplyPolicy::Unlimited),
            "Limited" => {
                let supply = supply.unwrap();
                Ok(SupplyPolicy::Limited { max: supply })
            }
            _ => Err(GutenError::UnsupportedSupply),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Reflect)]
pub struct Behaviours {
    pub composable: bool,
    pub loose: bool,
}

impl Behaviours {
    pub fn new(fields_vec: Vec<String>) -> Result<Behaviours, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let behaviours = Behaviours::fields();

        let field_struct = behaviours
            .iter()
            .map(|f| {
                let v = fields_to_add.contains(f);
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        Behaviours::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<Behaviours, GutenError> {
        let mut field_struct = Behaviours::default();

        for (f, v) in map {
            match f.as_str() {
                "composable" => {
                    field_struct.composable = *v;
                    Ok(())
                }
                "loose" => {
                    field_struct.loose = *v;
                    Ok(())
                }
                _ => Err(GutenError::UnsupportedNftBehaviour),
            }?;
        }

        Ok(field_struct)
    }

    pub fn fields() -> Vec<String> {
        let field_struct = Behaviours::default();
        let mut fields: Vec<String> = Vec::new();

        for (i, _) in field_struct.iter_fields().enumerate() {
            let field_name = field_struct.name_at(i).unwrap();

            fields.push(field_name.to_string());
        }
        fields
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Reflect)]
pub struct MintStrategy {
    pub launchpad: bool,
    pub airdrop: bool,
    pub direct: bool,
}

impl MintStrategy {
    pub fn new(fields_vec: Vec<String>) -> Result<MintStrategy, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let fields = MintStrategy::fields();

        let field_struct = fields
            .iter()
            .map(|f| {
                let v = fields_to_add.contains(f);
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        MintStrategy::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<MintStrategy, GutenError> {
        let mut field_struct = MintStrategy::default();

        for (f, v) in map {
            match f.as_str() {
                "launchpad" => {
                    field_struct.launchpad = *v;
                    Ok(())
                }
                "airdrop" => {
                    field_struct.airdrop = *v;
                    Ok(())
                }
                "direct" => {
                    field_struct.direct = *v;
                    Ok(())
                }
                _ => Err(GutenError::UnsupportedNftField),
            }?;
        }

        Ok(field_struct)
    }

    pub fn fields() -> Vec<String> {
        let field_struct = MintStrategy::default();
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

    pub fn write_domains(&self) -> String {
        let code = self
            .to_map()
            .iter()
            // Filter by domain fields set to true
            .filter(|(_, v)| *v)
            .map(|(k, _)| match k.as_str() {
                "display" => {
                    "        display::add_display_domain(
            &mut nft,
            name,
            description,
            ctx,
        );

"
                }
                "url" => {
                    "        display::add_url_domain(
            &mut nft,
            url::new_unsafe_from_bytes(url),
            ctx,
        );
"
                }
                "attributes" => {
                    "
        display::add_attributes_domain_from_vec(
            &mut nft,
            attribute_keys,
            attribute_values,
            ctx,
        );
"
                }
                "tags" => {
                    "
        tags::add_tag_domain(
            &mut nft,
            tags,
            ctx,
        );"
                }
                _ => {
                    eprintln!("File has no extension");
                    std::process::exit(2);
                }
            })
            .collect();

        code
    }

    pub fn write_mint_fn(
        &self,
        witness: &str,
        mint_strategy: MintType,
        nft_data: &NftData,
    ) -> String {
        let fun_name: String;
        let to_whom: String;
        let transfer: String;

        match mint_strategy {
            MintType::Direct => {
                fun_name = "direct_mint".to_string();
                to_whom = "        receiver: address,".to_string();
                transfer = "
            transfer::transfer(nft, receiver);"
                    .to_string();
            }
            MintType::Airdrop => {
                fun_name = "airdrop_mint".to_string();
                to_whom = format!(
                    "        mint_cap: &MintCap<{witness}>,
        receiver: address,",
                    witness = witness,
                );
                transfer = "
        transfer::transfer(nft, receiver);"
                    .to_string();
            }
            MintType::Launchpad => {
                fun_name = "mint_to_warehouse".to_string();
                to_whom = format!(
                    "        mint_cap: &MintCap<{witness}>,
        inventory: &mut Inventory,",
                    witness = witness,
                );
                transfer = "        inventory::deposit_nft(inventory, nft);"
                    .to_string();
            }
        };

        let domains = self.write_domains();
        let fields = nft_data.write_fields();

        let end_of_signature = format!(
            "        ctx: &mut TxContext,
    ) {{
        let nft = nft::new<{witness}>(tx_context::sender(ctx), ctx);\n",
            witness = witness
        );

        [
            format!(
                "
    public entry fun {fun_name}(",
                fun_name = fun_name
            ),
            fields,
            to_whom,
            end_of_signature,
            domains,
            transfer,
            "    }

            "
            .to_string(),
        ]
        .join("\n")
    }

    pub fn write_mint_fns(&self, name: &str, nft_data: &NftData) -> String {
        let mut mint_fns = String::new();

        if self.launchpad {
            mint_fns.push_str(&self.write_mint_fn(
                name,
                MintType::Launchpad,
                nft_data,
            ));
        }

        if self.airdrop {
            mint_fns.push_str(&self.write_mint_fn(
                name,
                MintType::Airdrop,
                nft_data,
            ));
        }

        if self.direct {
            mint_fns.push_str(&self.write_mint_fn(
                name,
                MintType::Direct,
                nft_data,
            ));
        }

        mint_fns
    }
}
