#[cfg(feature = "full")]
mod full {
    pub mod burn;
    pub mod dynamic;
    pub mod orderbook;

    pub use burn::Burn;
    pub use dynamic::Dynamic;
    pub use mint_cap::MintCap;
    pub use orderbook::Orderbook;
}

#[cfg(feature = "full")]
pub use full::*;

mod mint_cap;
mod minting;
mod request;

pub use minting::MintPolicies;
pub use request::RequestPolicies;

use super::collection::{CollectionData, Tags};
use serde::{Deserialize, Serialize};

// TODO: Merge `cfg(feature = "full")` and `cfg(not(feature = "full"))` definitions, requires manually
// implementing derives
#[cfg(feature = "full")]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    /// Type name of the NFT
    type_name: String,
    /// Burn policy for NFT
    burn: Burn,
    /// Dynamic policies for NFT
    dynamic: Dynamic,
    /// Mint capabilities issued for NFT
    mint_cap: MintCap,
    /// Additional mint functions to be generated for the NFT type such as
    /// Launchpad or Airdrop.
    mint_policies: MintPolicies,
    #[serde(default)]
    request_policies: RequestPolicies,
    #[serde(default)]
    orderbook: orderbook::Orderbook,
}

#[cfg(not(feature = "full"))]
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    /// Type name of the NFT
    type_name: String,
    /// Additional mint functions to be generated for the NFT type such as
    /// Launchpad or Airdrop.
    //
    // Custom mint policies are left as a non-full feature as it is a good
    // indicator of the integration problems that are solved by using
    // Gutenberg, therefore signifiying to the user the potential to save a ton
    // of time implementing integration boilerplate.
    mint_policies: MintPolicies,
    #[serde(default)]
    request_policies: RequestPolicies,
}

#[cfg(feature = "full")]
impl NftData {
    /// Create new [`NftData`]
    pub fn new(
        type_name: String,
        burn: Burn,
        dynamic: Dynamic,
        mint_cap: MintCap,
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
        orderbook: orderbook::Orderbook,
    ) -> Self {
        NftData {
            type_name,
            burn,
            dynamic,
            mint_cap,
            mint_policies,
            request_policies,
            orderbook,
        }
    }
}

#[cfg(not(feature = "full"))]
impl NftData {
    /// Create new [`NftData`]
    pub fn new(
        type_name: String,
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
    ) -> Self {
        NftData {
            type_name,
            mint_policies,
            request_policies,
        }
    }
}

impl NftData {
    /// Returns NFT type name
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Returns whether NFT requires withdraw policy to be created
    pub fn requires_withdraw(&self) -> bool {
        let requires_withdraw = self.request_policies.has_withdraw();
        #[cfg(feature = "full")]
        let requires_withdraw =
            requires_withdraw || self.burn.is_permissionless();
        requires_withdraw
    }

    /// Returns whether NFT requires borrow policy to be created
    pub fn requires_borrow(&self) -> bool {
        let requires_borrow = self.request_policies.has_borrow();
        #[cfg(feature = "full")]
        let requires_borrow = requires_borrow || self.dynamic.is_dynamic();
        requires_borrow
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
        let type_name = self.type_name();

        let collection_init = collection_data.write_move_init(self);
        let transfer_fns = self.write_move_transfer_fns();

        // Write MintCap instantiation
        //
        // If using non-full version of Gutenberg, a MintCap with supply
        // limited to 100 will always be instantiated
        #[cfg(feature = "full")]
        let mint_cap_init = self.mint_cap.write_move_init(&witness, type_name);
        #[cfg(not(feature = "full"))]
        let mint_cap_init = mint_cap::write_move_init(&witness, type_name);

        let mut misc_init = String::new();
        misc_init.push_str(&self.write_move_display(collection_data.tags()));
        misc_init.push_str(
            &self.write_move_policies(collection_data.has_royalties()),
        );
        #[cfg(feature = "full")]
        misc_init.push_str(&self.orderbook.write_move_init(type_name));

        format!(
            "

    fun init(witness: {witness}, ctx: &mut sui::tx_context::TxContext) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});{collection_init}{mint_cap_init}

        let publisher = sui::package::claim(witness, ctx);{misc_init}

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
        #[cfg(feature = "full")]
        defs_str.push_str(&self.dynamic.write_move_defs(type_name));
        #[cfg(feature = "full")]
        defs_str
            .push_str(&self.burn.write_move_defs(type_name, collection_data));
        defs_str
    }

    pub fn write_move_tests(&self, collection_data: &CollectionData) -> String {
        let type_name = self.type_name();

        #[allow(unused_mut)]
        let mut tests_str = String::new();
        tests_str.push_str(&self.write_mint_test(type_name, collection_data));
        #[cfg(feature = "full")]
        tests_str.push_str(
            &self.dynamic.write_move_tests(type_name, collection_data),
        );
        #[cfg(feature = "full")]
        tests_str
            .push_str(&self.burn.write_move_tests(type_name, collection_data));
        tests_str
    }

    fn write_mint_test(
        &self,
        type_name: &str,
        collection_data: &CollectionData,
    ) -> String {
        let witness = collection_data.witness_name();

        let requires_collection = collection_data.requires_collection();
        let collection_take_str = requires_collection.then(|| format!("

        let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<{type_name}>>(
            &scenario,
        );")).unwrap_or_default();

        let collection_param_str = requires_collection
            .then_some(
                "
            &mut collection,",
            )
            .unwrap_or_default();

        let collection_return_str = requires_collection
            .then_some(
                "
        sui::test_scenario::return_shared(collection);",
            )
            .unwrap_or_default();

        format!(
            "

    #[test]
    fun it_mints_nft() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );{collection_take_str}

        let warehouse = ob_launchpad::warehouse::new<{type_name}>(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_warehouse(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,{collection_param_str}
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);{collection_return_str}
        sui::test_scenario::end(scenario);
    }}")
    }

    fn write_move_policies(&self, has_royalties: bool) -> String {
        let type_name = self.type_name();

        let mut policies_str = String::new();

        if self.request_policies.has_transfer() {
            let royalty_strategy_str = has_royalties.then_some("
        nft_protocol::royalty_strategy_bps::enforce(&mut transfer_policy, &transfer_policy_cap);").unwrap_or_default();

            policies_str.push_str(&format!(
                "

        let (transfer_policy, transfer_policy_cap) = ob_request::transfer_request::init_policy<{type_name}>(
            &publisher, ctx,
        );{royalty_strategy_str}
        nft_protocol::transfer_allowlist::enforce(&mut transfer_policy, &transfer_policy_cap);"
            ));
        }

        if self.requires_borrow() {
            policies_str.push_str(&format!(
                "

        let (borrow_policy, borrow_policy_cap) = ob_request::borrow_request::init_policy<{type_name}>(
            &publisher, ctx,
        );"
            ));
        }

        if self.requires_withdraw() {
            policies_str.push_str(&format!(
                "

        let (withdraw_policy, withdraw_policy_cap) = ob_request::withdraw_request::init_policy<{type_name}>(
            &publisher, ctx,
        );"
            ));

            // When `NftData` requires a withdraw policy we must be careful to
            // protect it such that a malicious actor may not withdraw
            // arbitrarily
            if !self.request_policies.has_withdraw() {
                policies_str.push_str(&format!(
                    "
        ob_request::request::enforce_rule_no_state<ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>, Witness>(
            &mut withdraw_policy, &withdraw_policy_cap,
        );"
                ));
            }
        }

        policies_str
    }

    fn write_move_transfer_fns(&self) -> String {
        let mut code = String::new();

        if self.request_policies.has_transfer() {
            code.push_str(
                "

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);"
            );
        }

        if self.requires_withdraw() {
            code.push_str(
                "

        sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(withdraw_policy);"
            );
        }

        if self.requires_borrow() {
            code.push_str(
                "

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);"
            );
        }

        code
    }

    fn write_move_display(&self, tags: &Option<Tags>) -> String {
        let type_name = self.type_name();

        let tags_str = tags.as_ref().map(|tags| {
            let tags_str = tags.write_move_init();
            format!("{tags_str}

        sui::display::add(&mut display, std::string::utf8(b\"tags\"), ob_utils::display::from_vec(tags));")
        }).unwrap_or_default();

        format!("

        let display = sui::display::new<{type_name}>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b\"name\"), std::string::utf8(b\"{{name}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"description\"), std::string::utf8(b\"{{description}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"image_url\"), std::string::utf8(b\"{{url}}\"));
        sui::display::add(&mut display, std::string::utf8(b\"attributes\"), std::string::utf8(b\"{{attributes}}\"));{tags_str}
        sui::display::update_version(&mut display);

        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));"
        )
    }
}
