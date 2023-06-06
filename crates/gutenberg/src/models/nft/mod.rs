pub mod burn;
pub mod dynamic;

use burn::Burn;
use dynamic::Dynamic;
use serde::{Deserialize, Serialize};

use crate::contract::modules::Display;

use super::{collection::CollectionData, settings::Settings};

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    type_name: String,
    burn: Burn,
    dynamic: Dynamic,
}

impl NftData {
    pub fn new(type_name: String, burn: Burn, dynamic: bool) -> Self {
        NftData {
            type_name,
            burn,
            dynamic: dynamic.into(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Returns whether NFT requires withdraw policy to be created
    pub fn requires_withdraw(&self) -> bool {
        self.burn.is_permissionless()
    }

    /// Returns whether NFT requires borrow policy to be created
    pub fn requires_borrow(&self) -> bool {
        self.dynamic.is_dynamic()
    }

    fn write_move_struct(&self) -> String {
        let type_name = self.type_name();

        format!(
            "

    struct {type_name} has key, store {{
        id: sui::object::UID,
        name: std::string::String,
        description: std::string::String,
        url: sui::url::Url,
        attributes: nft_protocol::attributes::Attributes,
    }}"
        )
    }

    pub fn write_init_fn(
        &self,
        collection_data: &CollectionData,
        settings: &Settings,
    ) -> String {
        let type_name = self.type_name();
        let witness = collection_data.witness_name();

        let display = Display::write_display(type_name);

        let transfer_fns = self.write_transfer_fns(settings);
        let collection_init = collection_data.write_move_init();
        let settings_init = settings.write_move_init(self, collection_data);

        // Opt for `collection::create` over `collection::create_from_otw` in
        // order to statically assert `DelegatedWitness` gets created for the
        // `Collection<T>` type `T`.
        format!(
            "

    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});

        let collection = nft_protocol::collection::create<{type_name}>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);{collection_init}{settings_init}{display}

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(collection);{transfer_fns}
    }}"
        )
    }

    pub fn write_move_defs(
        &self,
        collection_data: &CollectionData,
        settings: &Settings,
    ) -> String {
        let type_name = self.type_name();

        let mut defs_str = String::new();
        defs_str.push_str(&self.write_move_struct());
        defs_str.push_str(&self.write_init_fn(collection_data, settings));
        defs_str.push_str(&self.dynamic.write_move_defs(type_name));
        defs_str
            .push_str(&self.burn.write_move_defs(type_name, collection_data));
        defs_str
    }

    pub fn write_move_tests(&self, collection_data: &CollectionData) -> String {
        let type_name = self.type_name();

        let mut defs_str = String::new();
        defs_str.push_str(
            &self.dynamic.write_move_tests(type_name, collection_data),
        );
        defs_str
            .push_str(&self.burn.write_move_tests(type_name, collection_data));
        defs_str
    }

    fn write_transfer_fns(&self, settings: &Settings) -> String {
        let mut code = String::new();

        if settings.request_policies().transfer {
            code.push_str(
                "

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);"
            );
        }

        if settings.request_policies().withdraw || self.requires_withdraw() {
            code.push_str(
                "

        sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(withdraw_policy);"
            );
        }

        if settings.request_policies().borrow || self.requires_borrow() {
            code.push_str(
                "

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);"
            );
        }

        code
    }
}
