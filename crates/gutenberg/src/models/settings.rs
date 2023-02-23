use std::collections::{BTreeSet, HashMap, HashSet};

use crate::{
    contract::modules::{ComposableNftMod, DisplayMod},
    err::GutenError,
    models::tags::Tags,
};

use bevy_reflect::{Reflect, Struct};
use serde::{Deserialize, Serialize};

use super::{
    collection::CollectionData,
    marketplace::{Listing, Listings},
    nft::NftData,
    royalties::RoyaltyPolicy,
    supply_policy::SupplyPolicy,
};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Settings {
    pub tags: Option<Tags>,               // Done
    pub royalties: Option<RoyaltyPolicy>, // Done
    #[serde(default)]
    pub mint_policies: MintPolicies,
    pub composability: Option<Composability>,
    #[serde(default)]
    pub loose: bool,
    pub listings: Option<Listings>,
}

impl Settings {
    pub fn new(
        tags: Option<Tags>,
        royalties: Option<RoyaltyPolicy>,
        mint_policies: MintPolicies,
        composability: Option<Composability>,
        loose: bool,
        listings: Option<Listings>,
    ) -> Settings {
        Settings {
            tags,
            royalties,
            mint_policies,
            composability,
            loose,
            listings,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_none()
            && self.royalties.is_none()
            && self.mint_policies.is_empty()
            && self.composability.is_none()
            && !self.loose
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

    pub fn write_transfer_fns(&self, receiver: Option<&String>) -> String {
        let receiver = match receiver {
            Some(address) => {
                if address == "sui::tx_context::sender(ctx)" {
                    address.clone()
                } else {
                    format!("@{address}")
                }
            }
            None => "sui::tx_context::sender(ctx)".to_string(),
        };

        let mut code = format!(
            "
        sui::transfer::transfer(mint_cap, {receiver});
        sui::transfer::share_object(collection);\n"
        );

        if self.loose {
            code.push_str(
                format!(
                    "        sui::transfer::transfer(templates, {receiver});"
                )
                .as_str(),
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
    types: BTreeSet<String>,
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
        types: BTreeSet<String>,
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

#[derive(Debug, Deserialize, Serialize, Reflect)]
pub struct MintPolicies {
    #[serde(default)]
    pub launchpad: bool,
    #[serde(default)]
    pub airdrop: bool,
    #[serde(default)]
    pub direct: bool,
}

impl Default for MintPolicies {
    fn default() -> Self {
        Self {
            launchpad: false,
            airdrop: true,
            direct: false,
        }
    }
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
                other => Err(GutenError::UnsupportedSettings(format!(
                    "The NFT mint policy provided `{}` is not supported",
                    other
                ))),
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
        collection: &CollectionData,
        nft: &NftData,
        mint_policy: Option<MintType>,
    ) -> String {
        let code: String;
        let witness = collection.witness_name();

        let mut return_type = String::new();
        let mut args = String::new();
        let mut domains = String::new();
        let mut params = String::new();
        let mut transfer = String::new();

        // Name and URL are mandatory as they are static display fields on the NFT
        args.push_str("        name: std::string::String,\n");
        params.push_str("            name,\n");

        args.push_str("        url: vector<u8>,\n");
        params.push_str("            url,\n");

        if nft.display {
            args.push_str(DisplayMod::add_display_args());
            domains.push_str(DisplayMod::add_nft_display());
            params.push_str(DisplayMod::add_display_params());
        }
        if nft.url {
            domains.push_str(DisplayMod::add_nft_url());
        }
        if nft.attributes {
            args.push_str(DisplayMod::add_attributes_args());
            domains.push_str(DisplayMod::add_nft_attributes());
            params.push_str(DisplayMod::add_attributes_params());
        }

        let mint_cap = match collection.supply_policy {
            SupplyPolicy::Unlimited => format!(
                "        mint_cap: &nft_protocol::mint_cap::UnregulatedMintCap<{witness}>,\n"
            ),
            SupplyPolicy::Limited { .. } => format!(
                "        mint_cap: &mut nft_protocol::mint_cap::RegulatedMintCap<{witness}>,\n"
            ),
            SupplyPolicy::Undefined => format!(
                "        mint_cap: &nft_protocol::mint_cap::MintCap<{witness}>,\n"
            ),
        };
        args.push_str(&mint_cap);

        params.push_str("            mint_cap,\n");
        params.push_str("            ctx,");

        if let Some(mint_policy) = mint_policy {
            let mut fun_name = String::new();

            match mint_policy {
                MintType::Launchpad => {
                    args.push_str(
                        format!(
                            "        warehouse: &mut nft_protocol::warehouse::Warehouse<{}>,\n",
                            witness
                        )
                        .as_str(),
                    );
                    transfer.push_str(
                        "nft_protocol::warehouse::deposit_nft(warehouse, nft);",
                    );
                    fun_name.push_str("mint_to_launchpad");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Airdrop => {
                    args.push_str("        receiver: address,\n");
                    transfer
                        .push_str("sui::transfer::transfer(nft, receiver);");
                    fun_name.push_str("mint_to_address");
                    args.push_str(
                        "        ctx: &mut sui::tx_context::TxContext,",
                    );
                }
                MintType::Direct => unimplemented!(),
            }

            code = format!(
                "\n
    public entry fun {fun_name}(
{args}
    ){return_type} {{
        let nft = mint(
{params}
        );

        {transfer}
    }}"
            );
        } else {
            return_type.push_str(
                format!(": nft_protocol::nft::Nft<{}>", witness).as_str(),
            );
            transfer.push_str("nft");

            args.push_str("        ctx: &mut sui::tx_context::TxContext,\n");

            let nft = match collection.supply_policy {
                SupplyPolicy::Unlimited => format!(
                    "let nft = nft_protocol::nft::from_unregulated(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
                SupplyPolicy::Limited { .. } => format!(
                    "let nft = nft_protocol::nft::from_regulated(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
                SupplyPolicy::Undefined => format!(
                    "let nft = nft_protocol::nft::from_mint_cap(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );"
                ),
            };

            code = format!(
                "\n
    fun mint(
{args}    ){return_type} {{
        {nft}
        let delegated_witness = nft_protocol::witness::from_witness<{witness}, Witness>(&Witness {{}});
{domains}
        {transfer}
    }}"
            );
        }

        code
    }

    pub fn write_mint_fns(
        &self,
        collection: &CollectionData,
        nft_data: &NftData,
    ) -> String {
        let mut mint_fns = String::new();

        if self.launchpad {
            mint_fns.push_str(&self.write_mint_fn(
                collection,
                nft_data,
                Some(MintType::Launchpad),
            ));
        }

        // TODO: For now the flow are indistinguishable
        if self.airdrop || self.direct {
            mint_fns.push_str(&self.write_mint_fn(
                collection,
                nft_data,
                Some(MintType::Airdrop),
            ));
        }

        mint_fns.push_str(&self.write_mint_fn(collection, nft_data, None));

        mint_fns
    }
}
