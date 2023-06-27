use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    str::FromStr,
};

use crate::{err::GutenError, models::collection::Supply};

use super::Fields;

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Burn {
    Permissioned,
    Permissionless,
}

impl Display for Burn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self {
            Burn::Permissioned => "Permissioned",
            Burn::Permissionless => "Permissionless",
        };

        f.write_str(string)
    }
}

impl FromStr for Burn {
    type Err = GutenError;

    fn from_str(level: &str) -> Result<Burn, Self::Err> {
        match level {
            "Permissioned" => Ok(Burn::Permissioned),
            "Permissionless" => Ok(Burn::Permissionless),
            level => Err(GutenError::UnsupportedSettings(
                format!("Burn level of `{level}` is unsupported. Supported levels include: [`None`, `Permissioned`, `Permissionless`].")
            ))
        }
    }
}

impl Burn {
    pub fn is_permissioned(&self) -> bool {
        matches!(self, Burn::Permissioned)
    }

    pub fn is_permissionless(&self) -> bool {
        matches!(self, Burn::Permissionless)
    }

    pub fn write_move_defs(
        &self,
        fields: &Fields,
        type_name: &str,
        requires_collection: bool,
        requires_listing: bool,
        requires_confirm: bool,
    ) -> String {
        let mut code = String::new();

        let mut_str = requires_collection.then_some("mut ").unwrap_or_default();

        let collection_decrement_str = requires_collection
            .then(|| Supply::write_move_decrement())
            .unwrap_or_default();

        let confirm_contract_str = requires_confirm
            .then_some(
                "
        ob_request::withdraw_request::add_receipt(&mut withdraw_request, &Witness {});"
            )
            .unwrap_or_default();

        let burn_nft_call = self
            .is_permissioned()
            .then_some("delegated_witness, ")
            .unwrap_or_default();

        let delegated_witness_init_str = self.is_permissionless()
            .then(|| format!(
            "
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});"
            ))
            .unwrap_or_default();

        let delegated_witness_publisher_init_str = self.is_permissioned()
            .then(|| format!(
            "
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);"
            ))
            .unwrap_or_default();

        let delegated_witness_param_str = self
            .is_permissioned()
            .then(|| {
                format!(
                    "
        delegated_witness: ob_permissions::witness::Witness<{type_name}>,"
                )
            })
            .unwrap_or_default();

        let publisher_param_str = self
            .is_permissioned()
            .then_some(
                "
        publisher: &sui::package::Publisher,",
            )
            .unwrap_or_default();

        let fields_str: String =
            fields.keys().map(|field| format!(", {field}: _")).collect();

        code.push_str(&format!(
            "

    public fun burn_nft({delegated_witness_param_str}
        collection: &{mut_str}nft_protocol::collection::Collection<{type_name}>,
        nft: {type_name},
    ) {{{delegated_witness_init_str}
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);
        let {type_name} {{ id{fields_str} }} = nft;
        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);{collection_decrement_str}
    }}

    public entry fun burn_nft_in_kiosk({delegated_witness_param_str}
        collection: &{mut_str}nft_protocol::collection::Collection<{type_name}>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);{confirm_contract_str}
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_nft({burn_nft_call}collection, nft);
    }}"
        ));

        if self.is_permissioned() {
            code.push_str(&format!("

    public entry fun burn_nft_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        collection: &{mut_str}nft_protocol::collection::Collection<{type_name}>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        burn_nft_in_kiosk(delegated_witness, collection, kiosk, nft_id, policy, ctx);
    }}"));
        }

        if requires_listing {
            code.push_str(&format!(
                "

    public entry fun burn_nft_in_listing({publisher_param_str}
        collection: &{mut_str}nft_protocol::collection::Collection<{type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{{delegated_witness_publisher_init_str}
        let nft = ob_launchpad::listing::admin_redeem_nft<{type_name}>(listing, inventory_id, ctx);
        burn_nft({burn_nft_call}collection, nft);
    }}

    public entry fun burn_nft_in_listing_with_id({publisher_param_str}
        collection: &{mut_str}nft_protocol::collection::Collection<{type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        nft_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{{delegated_witness_publisher_init_str}
        let nft = ob_launchpad::listing::admin_redeem_nft_with_id(listing, inventory_id, nft_id, ctx);
        burn_nft({burn_nft_call}collection, nft);
    }}"
            ));
        }

        code
    }

    pub fn write_move_tests(
        &self,
        fields: &Fields,
        type_name: &str,
        witness_name: &str,
        requires_collection: bool,
    ) -> String {
        let collection_param_str = requires_collection
            .then_some(
                "
            &mut collection,",
            )
            .unwrap_or_default();

        let collection_mut_str =
            requires_collection.then_some("mut ").unwrap_or_default();

        let delegated_witness_init_param_str = self
            .is_permissioned()
            .then(|| {
                format!(
                    "
                ob_permissions::witness::from_witness(Witness {{}}),"
                )
            })
            .unwrap_or_default();

        let fields_str: String = fields
            .test_params()
            .map(|param| {
                format!(
                    "
                {param},"
                )
            })
            .collect();

        format!("

        #[test]
        fun it_burns_nft() {{
            let scenario = sui::test_scenario::begin(CREATOR);
            init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));

            sui::test_scenario::next_tx(&mut scenario, CREATOR);

            let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
                &scenario,
                CREATOR,
            );

            let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
                &scenario,
                CREATOR,
            );

            let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<{type_name}>>(
                &scenario
            );

            let withdraw_policy = sui::test_scenario::take_shared<
                ob_request::request::Policy<
                    ob_request::request::WithNft<{type_name}, ob_request::withdraw_request::WITHDRAW_REQ>
                >
            >(&scenario);

            let nft = mint({fields_str}
                &mut mint_cap,{collection_param_str}
                sui::test_scenario::ctx(&mut scenario)
            );
            let nft_id = sui::object::id(&nft);

            let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
            ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

            burn_nft_in_kiosk({delegated_witness_init_param_str}
                &{collection_mut_str}collection,
                &mut kiosk,
                nft_id,
                &withdraw_policy,
                sui::test_scenario::ctx(&mut scenario)
            );

            sui::test_scenario::return_to_address(CREATOR, mint_cap);
            sui::test_scenario::return_to_address(CREATOR, publisher);
            sui::test_scenario::return_shared(collection);
            sui::test_scenario::return_shared(withdraw_policy);
            sui::transfer::public_share_object(kiosk);
            sui::test_scenario::end(scenario);
        }}")
    }
}
