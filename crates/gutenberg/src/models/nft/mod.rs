mod burn;
mod dynamic;
mod minting;

use burn::Burn;
use dynamic::Dynamic;
use minting::MintPolicies;
use serde::{Deserialize, Serialize};

use super::collection::CollectionData;

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    type_name: String,
    burn: Burn,
    dynamic: Dynamic,
    mint_policies: MintPolicies,
}

impl NftData {
    pub fn new(
        type_name: String,
        burn: Burn,
        dynamic: bool,
        mint_policies: MintPolicies,
    ) -> Self {
        NftData {
            type_name,
            burn,
            dynamic: dynamic.into(),
            mint_policies,
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

    pub fn write_init_fn(&self, collection_data: &CollectionData) -> String {
        let witness = collection_data.witness_name();

        let transfer_fns = self.write_transfer_fns(collection_data);
        let collection_init = collection_data.write_move_init(self);
        let display_init = self.write_move_display();

        // Opt for `collection::create` over `collection::create_from_otw` in
        // order to statically assert `DelegatedWitness` gets created for the
        // `Collection<T>` type `T`.
        format!(
            "

    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});{collection_init}{display_init}

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));{transfer_fns}
    }}"
        )
    }

    pub fn write_move_defs(&self, collection_data: &CollectionData) -> String {
        let type_name = self.type_name();

        let mut defs_str = String::new();
        defs_str.push_str(&self.write_move_struct());
        defs_str.push_str(&self.write_init_fn(collection_data));
        defs_str.push_str(
            &self
                .mint_policies
                .write_move_defs(type_name, collection_data),
        );
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

    fn write_transfer_fns(&self, collection_data: &CollectionData) -> String {
        let mut code = String::new();

        if collection_data.request_policies().has_transfer() {
            code.push_str(
                "

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);"
            );
        }

        if collection_data.request_policies().has_withdraw()
            || self.requires_withdraw()
        {
            code.push_str(
                "

        sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(withdraw_policy);"
            );
        }

        if collection_data.request_policies().has_borrow()
            || self.requires_borrow()
        {
            code.push_str(
                "

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);"
            );
        }

        code
    }

    fn write_move_display(&self) -> String {
        let type_name = self.type_name();

        format!("

        let display = sui::display::new<{type_name}>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b\"name\"), std::string::utf8(b\"{{name}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"description\"), std::string::utf8(b\"{{description}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"image_url\"), std::string::utf8(b\"{{url}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"attributes\"), std::string::utf8(b\"{{attributes}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"tags\"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);

        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));"
        )
    }
}
