use crate::models::collection;
use crate::MoveInit;
use crate::{models::write_move_fn, DefArgs, MoveDefs, MoveTests, TestArgs};
use gutenberg_types::models::nft::{Fields, MintPolicies};

impl MoveDefs for MintPolicies {
    fn write_move_defs(&self, args: DefArgs) -> String {
        let (fields, type_name, requires_collection) = def_args(args);

        let mut mint_fns = String::new();

        let params: Vec<String> = fields.params().collect();
        let mut base_params: Vec<&str> =
            params.iter().map(String::as_str).collect();
        base_params.push("mint_cap");

        if requires_collection {
            base_params.push("collection");
        }

        let mut nft_params = base_params.clone();
        nft_params.push("ctx");

        let mut base_param_types: Vec<&str> = fields.param_types().collect();
        let mint_cap_param =
            format!("&mut nft_protocol::mint_cap::MintCap<{type_name}>");
        base_param_types.push(&mint_cap_param);

        let collection_param =
            format!("&mut nft_protocol::collection::Collection<{type_name}>");
        if requires_collection {
            base_param_types.push(&collection_param);
        }

        let mut nft_param_types = base_param_types.clone();
        nft_param_types.push("&mut sui::tx_context::TxContext");

        // Mint NFT to Warehouse
        //
        // TODO: Mint NFT to Listing
        if self.launchpad {
            let mut params = base_params.clone();
            params.push("warehouse");
            params.push("ctx");

            let mut param_types = base_param_types.clone();
            let warehouse_param =
                format!("&mut ob_launchpad::warehouse::Warehouse<{type_name}>");
            param_types.push(&warehouse_param);
            param_types.push("&mut sui::tx_context::TxContext");

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_warehouse",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        ob_launchpad::warehouse::deposit_nft(warehouse, nft);",
                        params_str = nft_params.join(",\n            ")
                    )
                },
            ));
        }

        // Airdrop NFT into Kiosks
        if self.airdrop {
            // Write `mint_nft_to_kiosk`
            let mut params = base_params.clone();
            params.push("receiver");
            params.push("ctx");

            let mut param_types = base_param_types.clone();
            param_types.push("&mut sui::kiosk::Kiosk");
            param_types.push("&mut sui::tx_context::TxContext");

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_kiosk",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);",
                        params_str = nft_params.join(",\n            "),
                    )
                },
            ));

            // Write `mint_nft_to_new_kiosk`
            let mut params = base_params.clone();
            params.push("receiver");
            params.push("ctx");

            let mut param_types = base_param_types.clone();
            param_types.push("address");
            param_types.push("&mut sui::tx_context::TxContext");

            mint_fns.push_str(&write_move_fn(
                "mint_nft_to_new_kiosk",
                params.as_slice(),
                param_types.as_slice(),
                true,
                true,
                None,
                || {
                    format!(
                        "
        let nft = mint(
            {params_str},
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new_for_address(receiver, ctx);
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, ctx);
        sui::transfer::public_share_object(kiosk);",
                        params_str = nft_params.join(",\n            ")
                    )
                },
            ));
        }

        mint_fns.push_str(&write_move_fn(
            "mint",
            nft_params.as_slice(),
            nft_param_types.as_slice(),
            false,
            false,
            Some(type_name.to_string()),
            || {
                let collection_increment_str = requires_collection
                    .then(collection::supply::write_move_increment)
                    .unwrap_or_default();

                let fields_str: String = fields.iter().map(|field| {
                    let field_name = field.name();
                    let init = field
                        .write_move_init(None)
                        .map(|init| format!("{field_name}: {init}"))
                        .unwrap_or_else(|| field_name.to_string());

                    format!("
            {init},")
                }).collect();

                format!(
                    "
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});

        let nft = {type_name} {{
            id: sui::object::new(ctx),{fields_str}
        }};

        nft_protocol::mint_event::emit_mint(
            delegated_witness,
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );{collection_increment_str}

        nft_protocol::mint_cap::increment_supply(mint_cap, 1);

        nft",
                )
            },
        ));

        mint_fns
    }
}

impl MoveTests for MintPolicies {
    fn write_move_tests(&self, args: TestArgs) -> String {
        let (fields, type_name, witness_name, requires_collection) =
            test_args(args);

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

        let fields_str: String = fields
            .test_params()
            .map(|param| {
                format!(
                    "
            {param},"
                )
            })
            .collect();

        let mut test_str = String::new();

        if self.airdrop {
            test_str.push_str(&format!(
                "

    #[test]
    fun it_mints_nft_airdrop() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );{collection_take_str}

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_kiosk({fields_str}
            &mut mint_cap,{collection_param_str}
            &mut kiosk,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);{collection_return_str}
        sui::test_scenario::end(scenario);
    }}"));
        }

        if self.launchpad {
            test_str.push_str(&format!(
                "

    #[test]
    fun it_mints_nft_launchpad() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );{collection_take_str}

        let warehouse = ob_launchpad::warehouse::new<{type_name}>(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_warehouse({fields_str}
            &mut mint_cap,{collection_param_str}
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);{collection_return_str}
        sui::test_scenario::end(scenario);
    }}"));
        }

        test_str
    }
}

fn def_args(args: DefArgs) -> (&Fields, &str, bool) {
    match args {
        DefArgs::MintPolicies {
            fields,
            type_name,
            requires_collection,
        } => (fields, type_name, requires_collection),
        _ => panic!("Incorrect DefArgs variant"),
    }
}

fn test_args(args: TestArgs) -> (&Fields, &str, &str, bool) {
    match args {
        TestArgs::MintPolicies {
            fields,
            type_name,
            witness_name,
            requires_collection,
        } => (fields, type_name, witness_name, requires_collection),
        _ => panic!("Incorrect TestArgs variant"),
    }
}
