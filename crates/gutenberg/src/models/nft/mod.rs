#[cfg(feature = "full")]
mod burn;
#[cfg(feature = "full")]
mod dynamic;
#[cfg(feature = "full")]
mod mint_cap;
mod minting;
#[cfg(feature = "full")]
mod orderbook;
mod request;

#[cfg(feature = "full")]
pub use burn::Burn;
#[cfg(feature = "full")]
pub use dynamic::Dynamic;
#[cfg(feature = "full")]
pub use mint_cap::MintCap;
pub use minting::MintPolicies;
#[cfg(feature = "full")]
pub use orderbook::Orderbook;
pub use request::RequestPolicies;

use super::collection::{CollectionData, Tags};
use crate::normalize_type;
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
    #[serde(default)]
    burn: Burn,
    /// Dynamic policies for NFT
    #[serde(default)]
    dynamic: Dynamic,
    /// Mint capabilities issued for NFT
    mint_cap: MintCap,
    /// Additional mint functions to be generated for the NFT type such as
    /// Launchpad or Airdrop.
    #[serde(default)]
    mint_policies: MintPolicies,
    /// Additional request policies to be initialized for the NFT
    #[serde(default)]
    request_policies: RequestPolicies,
    /// Orderbook to be initialized for the NFT
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
    #[serde(default)]
    mint_policies: MintPolicies,
    /// Additional request policies to be initialized for the NFT
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
    pub fn type_name(&self) -> String {
        // Since `NftData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        normalize_type(&self.type_name)
    }

    pub fn module_name(&self) -> String {
        // Since `NftData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.type_name().to_lowercase()
    }

    pub fn witness_name(&self) -> String {
        // Since `NftData` can be deserialized from an untrusted source
        // it's fields must be escaped when preparing for display.
        self.type_name().to_uppercase()
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

    #[cfg(not(feature = "full"))]
    /// Write MintCap instantiation with supply always limited to 100
    fn write_mint_cap_init(witness: &str, type_name: &str) -> String {
        format!("

        let mint_cap = nft_protocol::mint_cap::new_limited<{witness}, {type_name}>(
            &witness, collection_id, 100, ctx
        );
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));")
    }

    pub fn write_init_fn(&self, collection_data: &CollectionData) -> String {
        let witness_name = self.witness_name();
        let type_name = self.type_name();

        let collection_init = collection_data.write_move_init(&type_name);
        let transfer_fns = self.write_move_transfer_fns();

        // Write MintCap instantiation
        //
        // If using non-full version of Gutenberg, a MintCap with supply
        // limited to 100 will always be instantiated
        #[cfg(feature = "full")]
        let mint_cap_init =
            self.mint_cap.write_move_init(&witness_name, &type_name);
        #[cfg(not(feature = "full"))]
        let mint_cap_init = NftData::write_move_init(&witness_name, &type_name);

        let mut misc_init = String::new();
        misc_init.push_str(&self.write_move_display(collection_data.tags()));
        misc_init.push_str(
            &self.write_move_policies(collection_data.has_royalties()),
        );
        #[cfg(feature = "full")]
        misc_init.push_str(&self.orderbook.write_move_init(&type_name));

        format!(
            "

    fun init(witness: {witness_name}, ctx: &mut sui::tx_context::TxContext) {{
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
                .write_move_defs(&type_name, collection_data),
        );
        #[cfg(feature = "full")]
        defs_str.push_str(&self.dynamic.write_move_defs(&type_name));
        #[cfg(feature = "full")]
        defs_str
            .push_str(&self.burn.write_move_defs(&type_name, collection_data));
        defs_str
    }

    pub fn write_move_tests(&self, collection_data: &CollectionData) -> String {
        let type_name = self.type_name();
        let witness_name = self.witness_name();

        #[allow(unused_mut)]
        let mut tests_str = String::new();
        tests_str.push_str(&self.mint_policies.write_mint_tests(
            &type_name,
            &witness_name,
            collection_data,
        ));
        #[cfg(feature = "full")]
        tests_str.push_str(&self.dynamic.write_move_tests(
            &type_name,
            &witness_name,
            collection_data,
        ));
        #[cfg(feature = "full")]
        tests_str.push_str(&self.burn.write_move_tests(
            &type_name,
            &witness_name,
            collection_data,
        ));
        tests_str
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
