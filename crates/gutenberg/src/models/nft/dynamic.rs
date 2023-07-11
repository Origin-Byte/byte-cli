use crate::{models::write_move_fn, DefArgs, MoveDefs, MoveTests, TestArgs};
use crate::{InitArgs, MoveInit};
use gutenberg_types::models::nft::{Dynamic, Fields};

impl MoveDefs for Dynamic {
    fn write_move_defs(&self, args: DefArgs) -> String {
        let (fields, type_name) = def_args(args);

        if !self.0 {
            return String::new();
        }

        fields
            .iter()
            .map(|field| {
                let field_name = field.name();
                write_move_field_setter(
                    type_name,
                    field_name,
                    field.params().collect(),
                    field.param_types().collect(),
                    field.write_move_init(InitArgs::None), // .unwrap_or_else(|| field_name.to_string()), # TODO: Double check
                )
            })
            .collect()
    }
}

impl MoveTests for Dynamic {
    fn write_move_tests(&self, args: TestArgs) -> String {
        let (fields, type_name, witness_name, requires_collection) =
            test_args(args);

        if !self.is_dynamic() {
            return String::new();
        }

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

        let setters_str: String = fields
            .iter()
            .map(|field| {
                let field_name = field.name();
                let field_init: String = field
                    .test_params()
                    .map(|init| {
                        format!(
                            "
            {init},"
                        )
                    })
                    .collect();

                format!(
                    "

        set_{field_name}_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,{field_init}
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );"
                )
            })
            .collect();

        // Nothing to test if there are no setters
        if setters_str.is_empty() {
            return String::new();
        }

        format!("

    #[test]
    fun it_sets_metadata() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &scenario,
            CREATOR,
        );{collection_take_str}

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario,
            CREATOR,
        );

        let borrow_policy: ob_request::request::Policy<ob_request::request::WithNft<{type_name}, ob_request::borrow_request::BORROW_REQ>> =
            sui::test_scenario::take_shared(&mut scenario);

        let nft = mint({fields_str}
            &mut mint_cap,{collection_param_str}
            sui::test_scenario::ctx(&mut scenario),
        );
        let nft_id = sui::object::id(&nft);

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));{setters_str}

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_to_address(CREATOR, publisher);
        sui::test_scenario::return_shared(borrow_policy);{collection_return_str}
        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::end(scenario);
    }}")
    }
}

fn write_move_field_setter(
    type_name: &str,
    field_name: &str,
    params: Vec<String>,
    param_types: Vec<&str>,
    init_str: String,
) -> String {
    let params: Vec<_> = params.iter().map(String::as_str).collect();
    let set_params_str = params.join(", ");

    let delegated_witness_param_type_str =
        format!("ob_permissions::witness::Witness<{type_name}>");
    let nft_param_type_str = format!("&mut {type_name}");

    let mut field_str = String::new();

    let mut set_params = Vec::new();
    set_params.extend_from_slice(&["_delegated_witness", "nft"]);
    set_params.extend_from_slice(params.as_slice());

    let mut set_param_types = Vec::new();
    set_param_types.push(delegated_witness_param_type_str.as_str());
    set_param_types.push(nft_param_type_str.as_str());
    set_param_types.extend_from_slice(param_types.as_slice());

    field_str.push_str(&write_move_fn(
        &format!("set_{field_name}"),
        &set_params,
        &set_param_types,
        true,
        false,
        None,
        || {
            format!(
                "
        nft.{field_name} = {init_str};"
            )
        },
    ));

    let mut set_params = Vec::new();
    set_params.extend_from_slice(&["publisher", "nft"]);
    set_params.extend_from_slice(params.as_slice());

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        "&sui::package::Publisher",
        nft_param_type_str.as_str(),
    ]);
    set_param_types.extend_from_slice(param_types.as_slice());

    field_str.push_str(&write_move_fn(
        &format!("set_{field_name}_as_publisher"),
        &set_params,
        &set_param_types,
        true,
        true,
        None,
        || {
            format!("
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_{field_name}(delegated_witness, nft, {set_params_str});")
        },
    ));

    let mut set_params = Vec::new();
    set_params.extend_from_slice(&["delegated_witness", "kiosk", "nft_id"]);
    set_params.extend_from_slice(params.as_slice());
    set_params.extend_from_slice(&["policy", "ctx"]);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        delegated_witness_param_type_str.as_str(),
        "&mut sui::kiosk::Kiosk",
        "sui::object::ID",
    ]);
    set_param_types.extend_from_slice(param_types.as_slice());
    let policy_param_str =
        format!("&ob_request::request::Policy<ob_request::request::WithNft<{type_name}, ob_request::borrow_request::BORROW_REQ>>");
    set_param_types.extend_from_slice(&[
        policy_param_str.as_str(),
        "&mut sui::tx_context::TxContext",
    ]);

    field_str.push_str(&write_move_fn(
        &format!("set_{field_name}_in_kiosk"),
        &set_params,
        &set_param_types,
        true,
        false,
        None,
        || {
            format!("
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<{type_name}>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut {type_name} = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_{field_name}(delegated_witness, nft, {set_params_str});

        ob_kiosk::ob_kiosk::return_nft<Witness, {type_name}>(kiosk, borrow, policy);")
        },
    ));

    let mut set_params = Vec::new();
    set_params.extend_from_slice(&["publisher", "kiosk", "nft_id"]);
    set_params.extend(params);
    set_params.extend_from_slice(&["policy", "ctx"]);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        "&sui::package::Publisher",
        "&mut sui::kiosk::Kiosk",
        "sui::object::ID",
    ]);
    set_param_types.extend_from_slice(param_types.as_slice());
    let policy_param_str =
        format!("&ob_request::request::Policy<ob_request::request::WithNft<{type_name}, ob_request::borrow_request::BORROW_REQ>>");
    set_param_types.extend_from_slice(&[
        policy_param_str.as_str(),
        "&mut sui::tx_context::TxContext",
    ]);

    field_str.push_str(&write_move_fn(
        &format!("set_{field_name}_in_kiosk_as_publisher"),
        &set_params,
        &set_param_types,
        true,
        true,
        None,
        || {
            format!("
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_{field_name}_in_kiosk(delegated_witness, kiosk, nft_id, {set_params_str}, policy, ctx);")
        },
    ));

    field_str
}

fn def_args(args: DefArgs) -> (&Fields, &str) {
    match args {
        DefArgs::Dynamic { fields, type_name } => (fields, type_name),
        _ => panic!("Incorrect DefArgs variant"),
    }
}

fn test_args(args: TestArgs) -> (&Fields, &str, &str, bool) {
    match args {
        TestArgs::Dynamic {
            fields,
            type_name,
            witness_name,
            requires_collection,
        } => (fields, type_name, witness_name, requires_collection),
        _ => panic!("Incorrect TestArgs variant"),
    }
}
