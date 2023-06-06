use serde::{Deserialize, Serialize};

use crate::models::collection::CollectionData;

#[derive(Debug, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfoMod();

impl DisplayInfoMod {
    pub fn add_collection_display_info(
        collection: &CollectionData,
    ) -> Option<String> {
        let name = collection.name();
        collection.description().as_ref().map(|description| {
            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::display_info::new(
                std::string::utf8(b\"{name}\"),
                std::string::utf8(b\"{description}\"),
            ),
        );"
            )
        })
    }

    pub fn add_collection_url(collection: &CollectionData) -> Option<String> {
        collection.url().as_ref().map(|url| {
            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            sui::url::new_unsafe_from_bytes(b\"{url}\"),
        );"
            )
        })
    }

    pub fn add_collection_symbol(
        collection: &CollectionData,
    ) -> Option<String> {
        collection.symbol().as_ref().map(|symbol| {
            format!(
                "

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::symbol::new(std::string::utf8(b\"{symbol}\")),
        );",
            )
        })
    }

    pub fn add_nft_display() -> &'static str {
        "

        nft_protocol::display::add_display_domain(
            delegated_witness, &mut nft, name, description,
        );"
    }

    pub fn add_nft_url() -> &'static str {
        "

        nft_protocol::display::add_url_domain(
            delegated_witness, &mut nft, sui::url::new_unsafe_from_bytes(url),
        );"
    }

    pub fn add_nft_attributes() -> &'static str {
        "

        nft_protocol::display::add_attributes_domain_from_vec(
            delegated_witness, &mut nft, attribute_keys, attribute_values,
        );"
    }

    pub fn params() -> Vec<String> {
        vec!["description", "url", "attribute_keys", "attribute_values"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    pub fn param_types() -> Vec<String> {
        vec![
            "std::string::String",
            "vector<u8>",
            "vector<std::ascii::String>",
            "vector<std::ascii::String>",
        ]
        .into_iter()
        .map(str::to_string)
        .collect()
    }
}

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
