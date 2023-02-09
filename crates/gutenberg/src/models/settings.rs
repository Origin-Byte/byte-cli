use std::collections::{HashMap, HashSet};

use crate::{
    err::{self, GutenError},
    models::tags::Tags,
};

use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};

use super::{
    collection::CollectionData,
    marketplace::{Listing, Listings},
    nft::NftData,
    royalties::RoyaltyPolicy,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub tags: Option<Tags>,               // Done
    pub royalties: Option<RoyaltyPolicy>, // Done
    pub mint_policies: MintPolicies,
    pub composability: Option<Composability>,
    pub loose: bool,
    pub supply_policy: SupplyPolicy,
    pub listings: Option<Listings>,
}

impl Settings {
    pub fn new(
        tags: Option<Tags>,
        royalties: Option<RoyaltyPolicy>,
        mint_policies: MintPolicies,
        composability: Option<Composability>,
        loose: bool,
        supply_policy: SupplyPolicy,
        listings: Option<Listings>,
    ) -> Settings {
        Settings {
            tags,
            royalties,
            mint_policies,
            composability,
            loose,
            supply_policy,
            listings,
        }
    }

    pub fn set_tags(&mut self, tags: Tags) {
        self.tags = Option::Some(tags);
    }

    pub fn set_royalties(&mut self, royalties: RoyaltyPolicy) {
        self.royalties = Option::Some(royalties);
    }

    pub fn set_mint_policies(&mut self, policies: MintPolicies) {
        self.mint_policies = policies;
    }

    pub fn set_composability(&mut self, composability: Composability) {
        self.composability = Option::Some(composability);
    }

    pub fn set_loose(&mut self, is_loose: bool) {
        self.loose = is_loose;
    }

    pub fn set_supply_policy(&mut self, supply: SupplyPolicy) {
        self.supply_policy = supply;
    }

    pub fn set_listings(&mut self, listings: Listings) {
        self.listings = Some(listings);
    }

    pub fn add_listing(&mut self, listing: Listing) {
        self.listings
            .get_or_insert_with(Default::default)
            .0
            .push(listing);
    }

    pub fn write_feature_domains(&self, collection: &CollectionData) -> String {
        let mut code = String::new();

        if let Some(_tags) = &self.tags {
            code.push_str(self.write_tags().as_str());
        }

        if let Some(_royalties) = &self.royalties {
            code.push_str(self.write_royalties().as_str());
        }

        if let Some(_composability) = &self.composability {
            code.push_str(self.write_composability().as_str());
        }

        if self.loose {
            code.push_str(self.write_loose(collection).as_str());
        }

        match self.supply_policy {
            SupplyPolicy::Limited { max: _, frozen: _ } => code.push_str(
                self.supply_policy
                    .write_domain()
                    // It's safe to unwrap we are checking
                    // that the policy is Limited
                    .unwrap()
                    .as_str(),
            ),
            _ => {}
        }

        code
    }

    pub fn write_init_listings(&self) -> String {
        let code = self
            .listings
            .iter()
            .flat_map(|listings| listings.0.iter())
            .map(Listing::write_init)
            .collect::<Vec<_>>();

        code.join("\n")
    }

    pub fn write_transfer_fns(&self) -> String {
        let mut code = String::from(
            "
            transfer::transfer(mint_cap, tx_context::sender(ctx));
            transfer::share_object(collection);\n",
        );

        if self.loose {
            code.push_str(
                "transfer::transfer(templates, tx_context::sender(ctx));",
            )
        }

        code
    }

    pub fn write_tags(&self) -> String {
        self.tags
            .as_ref()
            .expect("No collection tags setup found")
            .write_domain(true)
    }

    pub fn write_royalties(&self) -> String {
        self.royalties
            .as_ref()
            .expect("No collection royalties setup found")
            .write_domain()
    }

    pub fn write_composability(&self) -> String {
        self.composability
            .as_ref()
            .expect("No collection composability setup found")
            .write_domain()
    }

    pub fn write_loose(&self, collection: &CollectionData) -> String {
        format!(
            "let templates = templates::new_templates<{name}>(
                ctx,
            );\n",
            name = collection.name
        )
    }

    pub fn write_type_declarations(&self) -> String {
        match &self.composability {
            Some(composability) => composability.write_types(),
            None => "".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Composability {
    types: Vec<String>,
    blueprint: HashMap<String, Child>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Child {
    pub child_type: String,
    pub order: u64,
    pub limit: u64,
}

impl Child {
    pub fn new(child_type: String, order: u64, limit: u64) -> Self {
        Child {
            child_type,
            order,
            limit,
        }
    }
}

impl Composability {
    pub fn new_from_tradeable_traits(
        types: Vec<String>,
        core_trait: String,
    ) -> Self {
        let mut traits_ = types.clone();
        traits_.retain(|trait_| trait_ != &core_trait);

        let mut blueprint = HashMap::new();
        let mut i = 1;
        for trait_ in traits_.iter() {
            blueprint
                .insert(core_trait.clone(), Child::new(trait_.clone(), i, 1));

            i += 1;
        }

        Composability { types, blueprint }
    }

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
            .for_each(drop);

        types
    }

    pub fn write_domain(&self) -> String {
        let mut code =
            String::from("let blueprint = c_nft::new_blueprint(ctx);\n");

        self.blueprint
            .iter()
            .map(|(parent_type, child)| {
                code.push_str(
                    format!(
                        "c_nft::add_relationship<{parent_type}, {child_type}>(
                    &mut blueprint,
                    {limit}, // limit
                    {order}, // order
                    ctx
                );\n",
                        parent_type = parent_type,
                        child_type = child.child_type,
                        limit = child.limit,
                        order = child.order,
                    )
                    .as_str(),
                );
            })
            .for_each(drop);

        code.push_str("c_nft::add_blueprint_domain(delegated_witness, &mut collection, blueprint);\n");

        code
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
    Limited { max: u64, frozen: bool },
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
        max: Option<u64>,
        frozen: Option<bool>,
    ) -> Result<SupplyPolicy, GutenError> {
        match input {
            "Unlimited" => Ok(SupplyPolicy::Unlimited),
            "Limited" => {
                let max = max.unwrap();
                let frozen = frozen.unwrap();
                Ok(SupplyPolicy::Limited { max, frozen })
            }
            _ => Err(GutenError::UnsupportedSupply),
        }
    }

    pub fn write_domain(&self) -> Result<String, GutenError> {
        match self {
            SupplyPolicy::Limited { max, frozen } => {
                Ok(format!(
                    "supply_domain::regulate(
                        delegated_witness,
                        &mut collection
                        {max},
                        {frozen},
                        ctx,
                    );\n",
                    max = max,
                    frozen = frozen,
                ))
            },
            _ => Err(err::contextualize(
                "Error: Trying to write Supply domain when supply policy is not limited".to_string())
            ),
        }
    }
}

// #[derive(Debug, Deserialize, Serialize, Default, Reflect)]
// pub struct Behaviours {
//     pub composable: bool,
//     pub loose: bool,
// }

// impl Behaviours {
//     pub fn new(fields_vec: Vec<String>) -> Result<Behaviours, GutenError> {
//         let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

//         let behaviours = Behaviours::fields();

//         let field_struct = behaviours
//             .iter()
//             .map(|f| {
//                 let v = fields_to_add.contains(f);
//                 (f.clone(), v)
//             })
//             .collect::<Vec<(String, bool)>>();

//         Behaviours::from_map(&field_struct)
//     }

//     fn from_map(map: &Vec<(String, bool)>) -> Result<Behaviours, GutenError> {
//         let mut field_struct = Behaviours::default();

//         for (f, v) in map {
//             match f.as_str() {
//                 "composable" => {
//                     field_struct.composable = *v;
//                     Ok(())
//                 }
//                 "loose" => {
//                     field_struct.loose = *v;
//                     Ok(())
//                 }
//                 _ => Err(GutenError::UnsupportedNftBehaviour),
//             }?;
//         }

//         Ok(field_struct)
//     }

//     pub fn fields() -> Vec<String> {
//         let field_struct = Behaviours::default();
//         let mut fields: Vec<String> = Vec::new();

//         for (i, _) in field_struct.iter_fields().enumerate() {
//             let field_name = field_struct.name_at(i).unwrap();

//             fields.push(field_name.to_string());
//         }
//         fields
//     }
// }

#[derive(Debug, Deserialize, Serialize, Default, Reflect)]
pub struct MintPolicies {
    pub launchpad: bool,
    pub airdrop: bool,
    pub direct: bool,
}

impl MintPolicies {
    pub fn new(fields_vec: Vec<String>) -> Result<MintPolicies, GutenError> {
        let fields_to_add: HashSet<String> = HashSet::from_iter(fields_vec);

        let fields = MintPolicies::fields();

        let field_struct = fields
            .iter()
            .map(|f| {
                let v = fields_to_add.contains(f);
                (f.clone(), v)
            })
            .collect::<Vec<(String, bool)>>();

        MintPolicies::from_map(&field_struct)
    }

    fn from_map(map: &Vec<(String, bool)>) -> Result<MintPolicies, GutenError> {
        let mut field_struct = MintPolicies::default();

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
        let field_struct = MintPolicies::default();
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
        mint_policy: MintType,
        nft_data: &NftData,
    ) -> String {
        let fun_name: String;
        let to_whom: String;
        let transfer: String;

        match mint_policy {
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
