use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashSet;

use crate::prelude::GutenError;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub fields: Fields,
    pub behaviours: Behaviours,
    pub supply_policy: SupplyPolicy,
    pub mint_strategy: MintStrategy,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplyPolicy {
    Unlimited,
    Limited { max: u64 },
    Undefined,
}

impl SupplyPolicy {
    pub fn new_from(
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

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct Behaviours {
    composable: bool,
    loose: bool,
}

impl Behaviours {
    pub fn new() -> Behaviours {
        Behaviours {
            composable: false,
            loose: false,
        }
    }

    pub fn new_from(fields_vec: Vec<String>) -> Result<Behaviours, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let behaviours = Behaviours::fields();

        let field_struct = behaviours
            .iter()
            .map(|f| {
                let v = if fields_to_add.contains(f) {
                    true
                } else {
                    false
                };
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        Behaviours::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<Behaviours, GutenError> {
        let mut field_struct = Behaviours::new();

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
        let field_struct = Behaviours::new();
        let mut fields: Vec<String> = Vec::new();

        for (i, _) in field_struct.iter_fields().enumerate() {
            let field_name = field_struct.name_at(i).unwrap();

            fields.push(field_name.to_string());
        }
        fields
    }
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct Fields {
    display: bool,
    url: bool,
    attributes: bool,
    tags: bool,
}

impl Fields {
    pub fn new() -> Fields {
        Fields {
            display: false,
            url: false,
            attributes: false,
            tags: false,
        }
    }

    pub fn new_from(fields_vec: Vec<String>) -> Result<Fields, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let fields = Fields::fields();

        let field_struct = fields
            .iter()
            .map(|f| {
                let v = if fields_to_add.contains(f) {
                    true
                } else {
                    false
                };
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        Fields::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<Fields, GutenError> {
        let mut field_struct = Fields::new();

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
                _ => Err(GutenError::UnsupportedNftField),
            }?;
        }

        Ok(field_struct)
    }

    pub fn fields() -> Vec<String> {
        let field_struct = Fields::new();
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
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct MintStrategy {
    launchpad: bool,
    airdrop: bool,
    direct: bool,
}

impl MintStrategy {
    pub fn new() -> MintStrategy {
        MintStrategy {
            launchpad: false,
            airdrop: false,
            direct: false,
        }
    }

    pub fn new_from(
        fields_vec: Vec<String>,
    ) -> Result<MintStrategy, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let fields = MintStrategy::fields();

        let field_struct = fields
            .iter()
            .map(|f| {
                let v = if fields_to_add.contains(f) {
                    true
                } else {
                    false
                };
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        MintStrategy::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<MintStrategy, GutenError> {
        let mut field_struct = MintStrategy::new();

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
        let field_struct = MintStrategy::new();
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
}

pub enum Mint {
    Direct,
    Airdrop,
    Launchpad,
}

impl Nft {
    pub fn new() -> Nft {
        Nft {
            fields: Fields::new(),
            behaviours: Behaviours::new(),
            supply_policy: SupplyPolicy::Undefined,
            mint_strategy: MintStrategy::new(),
        }
    }

    pub fn write_domains(&self) -> String {
        let code = self
            .fields
            .to_map()
            .iter()
            .filter(|(_, v)| *v == true)
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

    pub fn write_fields(&self) -> String {
        // let s = serde_json::to_string(&self.fields).unwrap();
        // let domains: BTreeMap<String, bool> = serde_json::from_str(&s).unwrap();

        let code = self
            .fields
            .to_map()
            .iter()
            .filter(|(_, v)| *v == true)
            .map(|(k, _)| match k.as_str() {
                "display" => {
                    "        name: String,
        description: String,"
                }
                "url" => {
                    "
        url: vector<u8>,"
                }
                "attributes" => {
                    "
        attribute_keys: vector<String>,
        attribute_values: vector<String>,"
                }
                "tags" => {
                    "
        tags: Tags,"
                }
                _ => {
                    eprintln!("Field not recognized");
                    std::process::exit(2);
                }
            })
            .collect();

        code
    }

    pub fn mint_fn(&self, witness: &str, mint_strategy: Mint) -> String {
        let fun_name: String;
        let to_whom: String;
        let transfer: String;

        match mint_strategy {
            Mint::Direct => {
                fun_name = "direct_mint".to_string();
                to_whom = "        receiver: address,".to_string();
                transfer = "
            transfer::transfer(nft, receiver);"
                    .to_string();
            }
            Mint::Airdrop => {
                fun_name = "airdrop_mint".to_string();
                to_whom = format!(
                    "        _mint_cap: &MintCap<{witness}>,
        receiver: address,",
                    witness = witness,
                );
                transfer = "
        transfer::transfer(nft, receiver);"
                    .to_string();
            }
            Mint::Launchpad => {
                fun_name = "mint_to_warehouse".to_string();
                to_whom = format!(
                    "        _mint_cap: &MintCap<{witness}>,
        inventory: &mut Inventory,",
                    witness = witness,
                );
                transfer = "        inventory::deposit_nft(inventory, nft);"
                    .to_string();
            }
        };

        let domains = self.write_domains();
        let fields = self.write_fields();

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

    pub fn mint_fns(&self, witness: &str) -> String {
        let s = serde_json::to_string(&self.mint_strategy).unwrap();
        let strategies: BTreeMap<String, bool> =
            serde_json::from_str(&s).unwrap();

        let code: String = strategies
            .iter()
            .filter(|(_, v)| **v == true)
            .map(|(k, _)| match k.as_str() {
                "direct" => self.mint_fn(witness, Mint::Direct),
                "airdrop" => self.mint_fn(witness, Mint::Airdrop),
                "launchpad" => self.mint_fn(witness, Mint::Launchpad),
                _ => {
                    eprintln!("Mint strategy not supported");
                    std::process::exit(2);
                }
            })
            .collect();

        code
    }
}
