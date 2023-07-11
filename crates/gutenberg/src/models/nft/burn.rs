use crate::{
    models::collection::supply::write_move_decrement, DefArgs, MoveDefs,
    MoveTests, TestArgs,
};
use gutenberg_types::models::nft::{Burn, Fields};

impl MoveDefs for Burn {
    fn write_move_defs(&self, args: DefArgs) -> String {
        let (
            fields,
            type_name,
            requires_collection,
            requires_listing,
            requires_confirm,
        ) = def_args(args);

        let mut code = String::new();

        let mut_str = requires_collection.then_some("mut ").unwrap_or_default();

        let collection_decrement_str = requires_collection
            .then(write_move_decrement)
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
            .then_some(
            "
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});"
            )
            .unwrap_or_default();

        let delegated_witness_publisher_init_str = self.is_permissioned()
            .then_some(
            "
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);"
            )
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
}

impl MoveTests for Burn {
    fn write_move_tests(&self, args: TestArgs) -> String {
        let (fields, type_name, witness_name, requires_collection) =
            test_args(args);

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
            .then_some(
                "
                ob_permissions::witness::from_witness(Witness {}),",
            )
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

fn def_args(args: DefArgs) -> (&Fields, &str, bool, bool, bool) {
    match args {
        DefArgs::Burn {
            fields,
            type_name,
            requires_collection,
            requires_listing,
            requires_confirm,
        } => (
            fields,
            type_name,
            requires_collection,
            requires_listing,
            requires_confirm,
        ),
        _ => panic!("Incorrect DefArgs variant"),
    }
}

fn test_args(args: TestArgs) -> (&Fields, &str, &str, bool) {
    match args {
        TestArgs::Burn {
            fields,
            type_name,
            witness_name,
            requires_collection,
        } => (fields, type_name, witness_name, requires_collection),
        _ => panic!("Incorrect TestArgs variant"),
    }
}
