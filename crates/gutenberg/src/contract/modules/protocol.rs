use serde::{Deserialize, Serialize};

use super::{sui::Url, Module};

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftMod();

impl Module for NftMod {
    fn import(&self) -> String {
        "use nft_protocol::nft;".to_string()
    }
}

impl NftMod {
    pub fn new(receiver: &str) -> String {
        format!("nft::new(&Witness {{}}, mint_cap, {}, ctx);", receiver)
    }

    pub fn new_to_sender() -> String {
        "nft::new(&Witness {{}}, mint_cap, tx_context::sender(ctx), ctx);"
            .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TagsMod();

impl Module for TagsMod {
    fn import(&self) -> String {
        "use nft_protocol::tags;".to_string()
    }
}

impl TagsMod {
    pub fn init_tags() -> String {
        "let tags = tags::empty(ctx);".to_string()
    }

    pub fn add_tag(tag: &str) -> String {
        format!("tags::add_tag(&mut tags, tags::{}());", tag)
    }

    pub fn add_collection_domain() -> String {
        "tags::add_collection_tag_domain(
            delegated_witness,
            &mut collection,
            tags,
        );"
        .to_string()
    }

    pub fn add_nft_domain() -> String {
        "tags::add_tag_domain(
            delegated_witness,
            &mut nft,
            tags,
        );"
        .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoyaltyMod();

impl Module for RoyaltyMod {
    fn import(&self) -> String {
        "use nft_protocol::tags;".to_string()
    }
}

impl RoyaltyMod {
    pub fn import() -> String {
        "use nft_protocol::royalty;".to_string()
    }

    pub fn from_sender_address() -> String {
        "royalty::from_address(tx_context::sender(ctx), ctx);".to_string()
    }

    pub fn add_proportional(bps: u64) -> String {
        format!("royalty::add_proportional_royalty(&mut royalty, {});", bps)
    }

    pub fn add_constant(fee: u64) -> String {
        format!("royalty::add_constant_royalty(&mut royalty, {});", fee)
    }

    pub fn add_collection_domain() -> String {
        "royalty::add_royalty_domain(
            delegated_witness,
            &mut collection,
            royalty,
        );"
        .to_string()
    }

    pub fn get_domain() -> String {
        "royalty::royalty_domain(collection);".to_string()
    }

    pub fn calculate_proportional_royalty(
        domain: &str,
        balance: &str,
    ) -> String {
        format!(
            "royalty::calculate_proportional_royalty({}, balance::value({}));",
            domain, balance,
        )
    }

    pub fn calculate_constant_royalty(domain: &str) -> String {
        format!("royalty::calculate_constant_royalty({});", domain)
    }

    pub fn collect_royalty(balance: &str, royalty_owed: &str) -> String {
        format!(
            "royalty::collect_royalty(collection, {}, {});",
            balance, royalty_owed,
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisplayMod();

impl Module for DisplayMod {
    fn import(&self) -> String {
        "use nft_protocol::display;".to_string()
    }
}

impl DisplayMod {
    pub fn add_collection_display(name: &str, description: &str) -> String {
        format!(
            "display::add_collection_display_domain(
                delegated_witness,
                &mut collection,
                string::utf8(b\"{}\"),
                string::utf8(b\"{}\"),
                ctx,
            );",
            name, description
        )
    }

    pub fn add_collection_url(url: &str) -> String {
        format!(
            "display::add_collection_url_domain(
                delegated_witness,
                &mut collection,
                {url}
                ctx,
            );",
            url = Url::to_url_param(url, false)
        )
    }

    pub fn add_collection_symbol(symbol: &str) -> String {
        format!(
            "display::add_collection_symbol_domain(
                delegated_witness,
                &mut collection,
                string::utf8(b\"{}\"),
                ctx
            );",
            symbol
        )
    }

    pub fn add_nft_display(name: &str, description: &str) -> String {
        format!(
            "display::add_display_domain(
                delegated_witness, &mut nft, {}, {}, ctx,
            );",
            name, description
        )
    }

    pub fn add_nft_url() -> String {
        "display::add_url_domain(
            delegated_witness, &mut nft, url::new_unsafe_from_bytes(url), ctx,
        );"
        .to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WitnessMod();

impl Module for WitnessMod {
    fn import(&self) -> String {
        "use nft_protocol::witness;".to_string()
    }
}

impl WitnessMod {
    pub fn get_delegated_witness() -> String {
        "witness::from_witness(&Witness {});".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreatorsMod();

impl Module for CreatorsMod {
    fn import(&self) -> String {
        "use nft_protocol::creators;".to_string()
    }
}

impl CreatorsMod {
    pub fn from_sender_address(otw: &str) -> String {
        format!(
            "creators::from_address<{otw}, Witness>(
            &Witness {{}}, tx_context::sender(ctx), ctx,
        ),",
            otw = otw
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WarehouseMod();

impl Module for WarehouseMod {
    fn import(&self) -> String {
        "use nft_protocol::warehouse::{Self, Warehouse};".to_string()
    }
}

impl WarehouseMod {
    pub fn deposit_nft() -> String {
        "warehouse::deposit_nft(warehouse, nft);".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoyaltiesMod();

impl Module for RoyaltiesMod {
    fn import(&self) -> String {
        "use nft_protocol::royalties::{Self, TradePayment};".to_string()
    }
}

impl RoyaltiesMod {
    pub fn balance_mut() -> String {
        "royalties::balance_mut(Witness {}, payment);".to_string()
    }

    pub fn transfer_remaining_to_beneficiary() -> String {
        "royalties::transfer_remaining_to_beneficiary(Witness {}, payment, ctx);".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMod();

impl Module for CollectionMod {
    fn import(&self) -> String {
        "use nft_protocol::collection::{Self, Collection};".to_string()
    }
}

impl CollectionMod {
    pub fn create() -> String {
        "collection::create(&witness, ctx);".to_string()
    }

    pub fn add_domain(domain: &str) -> String {
        format!(
            "collection::add_domain(
                delegated_witness,
                &mut collection,
                {domain},
            );",
            domain = domain
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MintCapMod();

impl Module for MintCapMod {
    fn import(&self) -> String {
        "use nft_protocol::mint_cap::{MintCap};".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComposableNftMod();

impl Module for ComposableNftMod {
    fn import(&self) -> String {
        "use nft_protocol::composable_nft::Self as c_nft;".to_string()
    }
}

impl ComposableNftMod {
    pub fn add_type(type_name: &str) -> String {
        format!(
            "struct {type_name} has copy, drop, store {{}}",
            type_name = type_name
        )
    }

    pub fn init_blueprint() -> String {
        "let blueprint = c_nft::new_blueprint(ctx);".to_string()
    }

    pub fn add_parent_child_relationship(
        parent_type: &str,
        child_type: &str,
    ) -> String {
        format!(
            "c_nft::add_parent_child_relationship<{parent_type}>(
                &mut blueprint,
                c_nft::new_child_node<{child_type}>(1, 1, ctx), // limit, order, ctx
                ctx
            );",
            parent_type = parent_type,
            child_type = child_type,
        )
    }

    pub fn add_collection_domain() -> String {
        "c_nft::add_blueprint_domain(delegated_witness, &mut collection, blueprint);".to_string()
    }

    pub fn add_type_to_nft(otw: &str) -> String {
        format!("c_nft::add_type_domain<{otw}, T>(delegated_witness, &mut nft, ctx);", otw = otw)
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplateMod();

impl Module for TemplateMod {
    fn import(&self) -> String {
        "use nft_protocol::template;".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TemplatesMod();

impl Module for TemplatesMod {
    fn import(&self) -> String {
        "use nft_protocol::templates;".to_string()
    }
}
