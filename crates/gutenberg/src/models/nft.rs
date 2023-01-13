use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    fields: Fields,
    behaviours: Behaviours,
    supply_policy: SupplyPolicy,
    mint_strategy: MintStrategy,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SupplyPolicy {
    Unlimited,
    Limited { max: u64 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Behaviours {
    composable: bool,
    loose: bool,
}

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct Fields {
    display: bool,
    url: bool,
    attributes: bool,
    tags: bool,
}

impl Fields {
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

#[derive(Debug, Deserialize, Serialize)]
pub struct MintStrategy {
    direct: bool,
    airdrop: bool,
    launchpad: bool,
}

pub enum Mint {
    Direct,
    Airdrop,
    Launchpad,
}

impl Nft {
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
