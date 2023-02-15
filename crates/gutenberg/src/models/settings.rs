use std::collections::{HashMap, HashSet};

use crate::{
    contract::modules::{ComposableNftMod, DisplayMod},
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

    pub fn is_empty(&self) -> bool {
        self.tags.is_none()
            && self.royalties.is_none()
            && self.mint_policies.is_empty()
            && self.composability.is_none()
            && !self.loose
            && self.supply_policy.is_empty()
            && self.listings.is_none()
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
            "transfer::transfer(mint_cap, tx_context::sender(ctx));
        transfer::share_object(collection);\n",
        );

        if self.loose {
            code.push_str(
                "        transfer::transfer(templates, tx_context::sender(ctx));",
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
            "\n        let templates = templates::new_templates<{witness}>(
                ctx,
            );\n",
            witness = collection.witness_name(),
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
                types.push_str(ComposableNftMod::add_type(t).as_str());
            })
            .for_each(drop);

        types
    }

    pub fn write_domain(&self) -> String {
        let mut code = ComposableNftMod::init_blueprint();

        self.blueprint
            .iter()
            .map(|(parent_type, child)| {
                code.push_str(
                    format!(
                        "
        c_nft::add_relationship<{parent_type}, {child_type}>(
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

        code.push_str(ComposableNftMod::add_collection_domain().as_str());

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

    pub fn is_empty(&self) -> bool {
        matches!(self, SupplyPolicy::Undefined)
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

    pub fn is_empty(&self) -> bool {
        !self.launchpad && !self.airdrop && !self.direct
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

    pub fn write_mint_fn(
        &self,
        witness: &str,
        mint_policy: Option<MintType>,
        nft: &NftData,
    ) -> String {
        let code: String;

        let mut fun_name = String::new();
        let mut fun_type = String::new();
        let mut return_type = String::new();
        let mut args = String::new();
        let mut domains = String::new();
        let mut params = String::new();
        let mut transfer = String::new();

        if nft.display {
            args.push_str(DisplayMod::add_display_args().as_str());
            domains.push_str(DisplayMod::add_nft_display().as_str());
            params.push_str(DisplayMod::add_display_params().as_str());
        }
        if nft.url {
            args.push_str(DisplayMod::add_url_args().as_str());
            domains.push_str(DisplayMod::add_nft_url().as_str());
            params.push_str(DisplayMod::add_url_params().as_str());
        }
        if nft.attributes {
            args.push_str(DisplayMod::add_attributes_args().as_str());
            domains.push_str(DisplayMod::add_nft_attributes().as_str());
            params.push_str(DisplayMod::add_attributes_params().as_str());
        }

        args.push_str("        mint_cap: &MintCap<SUIMARINES>,\n");

        params.push_str("            mint_cap,\n");
        params.push_str("            ctx,");

        if let Some(mint_policy) = mint_policy {
            match mint_policy {
                MintType::Launchpad => {
                    args.push_str(
                        format!(
                            "        warehouse: &mut Warehouse<{}>,",
                            witness
                        )
                        .as_str(),
                    );
                    transfer
                        .push_str("warehouse::deposit_nft(warehouse, nft);");
                    fun_name.push_str("mint_to_launchpad");
                    args.push_str("        ctx: &mut TxContext,\n");
                }
                _ => {
                    args.push_str("        receiver: address,");
                    transfer.push_str("transfer::transfer(nft, receiver);");
                    fun_name.push_str("mint_to_address");
                    args.push_str("        ctx: &mut TxContext,\n");
                }
            }

            fun_type.push_str("public entry ");

            code = format!(
                "\n
    {fun_type}fun {fun_name}(
        {args}
    ){return_type} {{
        let nft = mint(
            {params}
        );

        {transfer}
    }}"
            );
        } else {
            fun_name.push_str("mint");
            return_type.push_str(format!(": Nft<{}>", witness).as_str());
            transfer.push_str("nft");

            let build_nft = "let nft =
            nft::new(&Witness {}, mint_cap, tx_context::sender(ctx), ctx);
        let delegated_witness = witness::from_witness(&Witness {});\n";

            args.push_str("        ctx: &mut TxContext,\n");

            code = format!(
                "\n
    {fun_type}fun {fun_name}(
        {args}        ){return_type} {{
        {build_nft}
        {domains}
        {transfer}
    }}"
            );
        }

        code
    }

    pub fn write_mint_fns(&self, witness: &str, nft_data: &NftData) -> String {
        let mut mint_fns = String::new();

        if self.launchpad {
            mint_fns.push_str(&self.write_mint_fn(
                witness,
                Some(MintType::Launchpad),
                nft_data,
            ));
        }

        // TODO: For now the flow are indistinguishable
        if self.airdrop || self.direct {
            mint_fns.push_str(&self.write_mint_fn(
                witness,
                Some(MintType::Airdrop),
                nft_data,
            ));
        }

        mint_fns.push_str(&self.write_mint_fn(witness, None, nft_data));

        mint_fns
    }
}
