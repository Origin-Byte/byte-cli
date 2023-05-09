use serde::{Deserialize, Serialize};

use crate::models::collection::CollectionData;
#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NftMod();

impl NftMod {
    pub fn new_to_sender() -> String {
        "let nft = nft_protocol::nft::from_mint_cap(mint_cap, name, sui::url::new_unsafe_from_bytes(url), ctx);".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoyaltyMod();

impl RoyaltyMod {
    pub fn from_sender_address() -> String {
        "royalty::from_address(tx_context::sender(ctx), ctx);\n".to_string()
    }

    pub fn add_proportional(bps: u64) -> String {
        format!(
            "royalty::add_proportional_royalty(&mut royalty, {});\n",
            bps
        )
    }

    pub fn add_constant(fee: u64) -> String {
        format!("royalty::add_constant_royalty(&mut royalty, {});\n", fee)
    }

    pub fn add_collection_domain() -> String {
        "royalty::add_royalty_domain(
            delegated_witness,
            &mut collection,
            royalty,
        );\n"
            .to_string()
    }

    pub fn get_domain() -> String {
        "royalty::royalty_domain(collection);\n".to_string()
    }

    pub fn calculate_proportional_royalty(
        domain: &str,
        balance: &str,
    ) -> String {
        format!(
            "royalty::calculate_proportional_royalty({}, balance::value({}));\n",
            domain, balance,
        )
    }

    pub fn calculate_constant_royalty(domain: &str) -> String {
        format!("royalty::calculate_constant_royalty({})\n;", domain)
    }

    pub fn collect_royalty(balance: &str, royalty_owed: &str) -> String {
        format!(
            "royalty::collect_royalty(collection, {}, {});\n",
            balance, royalty_owed,
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfoMod();

impl DisplayInfoMod {
    pub fn add_collection_display_info(
        collection: &CollectionData,
    ) -> Option<String> {
        collection.description.as_ref().map(|description| {
            format!(
                "
        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::display_info::new(
                std::string::utf8(b\"{collection_name}\"),
                std::string::utf8(b\"{description}\"),
            ),
        );\n",
                collection_name = collection.name
            )
        })
    }

    pub fn add_collection_url(collection: &CollectionData) -> Option<String> {
        collection.url.as_ref().map(|url| {
            format!(
                "
        nft_protocol::display::add_collection_url_domain(
            delegated_witness,
            &mut collection,
            {url}
        );\n",
                url = format!("sui::url::new_unsafe_from_bytes(b\"{}\"),", url)
            )
        })
    }

    pub fn add_collection_symbol(
        collection: &CollectionData,
    ) -> Option<String> {
        collection.symbol.as_ref().map(|symbol| {
            format!(
                "
        nft_protocol::display::add_collection_symbol_domain(
            delegated_witness,
            &mut collection,
            std::string::utf8(b\"{symbol}\"),
        );\n",
            )
        })
    }

    pub fn add_nft_display() -> &'static str {
        "
        nft_protocol::display::add_display_domain(
            delegated_witness, &mut nft, name, description,
        );\n"
    }

    pub fn add_nft_url() -> &'static str {
        "
        nft_protocol::display::add_url_domain(
            delegated_witness, &mut nft, sui::url::new_unsafe_from_bytes(url),
        );\n"
    }

    pub fn add_nft_attributes() -> &'static str {
        "
        nft_protocol::display::add_attributes_domain_from_vec(
            delegated_witness, &mut nft, attribute_keys, attribute_values,
        );\n"
    }

    pub fn add_display_args() -> &'static str {
        "        description: std::string::String,\n"
    }

    pub fn add_url_args() -> &'static str {
        "        url: vector<u8>,\n"
    }

    pub fn add_attributes_args() -> &'static str {
        "        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,\n"
    }

    pub fn add_display_params() -> &'static str {
        "            description,\n"
    }

    pub fn add_url_params() -> &'static str {
        "            url,\n"
    }

    pub fn add_attributes_params() -> &'static str {
        "            attribute_keys,
            attribute_values,\n"
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct WitnessMod();

impl WitnessMod {
    pub fn get_delegated_witness() -> String {
        "witness::from_witness(&Witness {});\n".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CreatorsMod();

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

impl WarehouseMod {
    pub fn deposit_nft() -> String {
        "warehouse::deposit_nft(warehouse, nft);\n".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RoyaltiesMod();

impl RoyaltiesMod {
    pub fn balance_mut() -> String {
        "royalties::balance_mut(Witness {}, payment);\n".to_string()
    }

    pub fn transfer_remaining_to_beneficiary() -> String {
        "royalties::transfer_remaining_to_beneficiary(Witness {}, payment, ctx);\n".to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMod();

impl CollectionMod {
    pub fn create() -> String {
        "nft_protocol::collection::create(&witness, ctx);\n".to_string()
    }

    pub fn add_domain(domain: &str) -> String {
        format!(
            "nft_protocol::collection::add_domain(
                delegated_witness,
                &mut collection,
                {domain},
            );\n",
            domain = domain
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct MintCapMod();

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ComposableNftMod();

impl ComposableNftMod {
    pub fn add_type(type_name: &str) -> String {
        format!(
            "
    struct {type_name} has copy, drop, store {{}}",
            type_name = type_name
        )
    }

    pub fn init_blueprint() -> String {
        "
        let blueprint = c_nft::new_blueprint(ctx);\n"
            .to_string()
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
            );\n",
            parent_type = parent_type,
            child_type = child_type,
        )
    }

    pub fn add_collection_domain() -> String {
        "        c_nft::add_blueprint_domain(delegated_witness, &mut collection, blueprint);\n".to_string()
    }

    pub fn add_type_to_nft(otw: &str) -> String {
        format!("c_nft::add_type_domain<{otw}, T>(delegated_witness, &mut nft, ctx);\n", otw = otw)
    }
}
