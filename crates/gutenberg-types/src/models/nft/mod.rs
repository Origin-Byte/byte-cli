mod burn;
mod dynamic;
mod fields;
mod mint_cap;
mod minting;
mod orderbook;
mod request;

use crate::normalize_type;
pub use burn::Burn;
pub use dynamic::Dynamic;
pub use fields::{Field, FieldType, Fields};
pub use mint_cap::MintCap;
pub use minting::MintPolicies;
pub use orderbook::Orderbook;
pub use request::RequestPolicies;
use serde::{Deserialize, Serialize};

// TODO: Merge `cfg(feature = "full")` and `cfg(not(feature = "full"))` definitions, requires manually
// implementing derives

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NftData {
    /// Type name of the NFT
    pub type_name: String,
    /// Burn policy for NFT
    pub burn: Option<Burn>,
    /// Dynamic policies for NFT
    #[serde(default)]
    pub dynamic: Dynamic,
    /// Mint capabilities issued for NFT
    pub mint_cap: MintCap,
    /// Additional mint functions to be generated for the NFT type such as
    /// Launchpad or Airdrop.
    #[serde(default)]
    pub mint_policies: MintPolicies,
    /// Additional request policies to be initialized for the NFT
    #[serde(default)]
    pub request_policies: RequestPolicies,
    /// Orderbook to be initialized for the NFT
    pub orderbook: Option<orderbook::Orderbook>,
    /// NFT fields and types
    #[serde(default)]
    pub fields: Fields,
}

impl NftData {
    /// Create new [`NftData`]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_name: String,
        burn: Option<Burn>,
        dynamic: Dynamic,
        mint_cap: MintCap,
        mint_policies: MintPolicies,
        request_policies: RequestPolicies,
        orderbook: Option<orderbook::Orderbook>,
        fields: Fields,
    ) -> Self {
        NftData {
            type_name,
            burn,
            dynamic,
            mint_cap,
            mint_policies,
            request_policies,
            orderbook,
            fields,
        }
    }

    /// Returns whether NFT requires transfer policy to be created
    pub fn requires_transfer(&self) -> bool {
        self.request_policies.has_transfer() || self.orderbook.is_some()
    }

    /// Returns whether NFT requires withdraw policy to be created
    pub fn requires_withdraw(&self) -> bool {
        self.request_policies.has_withdraw() || self.burn.is_some()
    }

    /// Returns whether NFT requires borrow policy to be created
    pub fn requires_borrow(&self) -> bool {
        self.request_policies.has_borrow() || self.dynamic.is_dynamic()
    }

    pub fn fields(&self) -> &Fields {
        &self.fields
    }

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

    /// Disables features that should not be enabled in demo mode
    pub fn enforce_demo(&mut self) {
        self.burn = None;
        self.dynamic = Dynamic::new(false);
        self.mint_cap = MintCap::limited(100);
        self.request_policies = RequestPolicies::new(false, false, false);
        self.orderbook = None;
        // Only allow a certain field configuration in demo mode
        self.fields = vec![
            ("name", FieldType::String),
            ("description", FieldType::String),
            ("url", FieldType::Url),
            ("attributes", FieldType::Attributes),
        ]
        .into()
    }
}
