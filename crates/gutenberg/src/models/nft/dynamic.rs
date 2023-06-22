use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

use crate::models::write_move_fn;

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone)]
#[serde(transparent)]
pub struct Dynamic(bool);

impl Default for Dynamic {
    /// Static NFT by default is a reasonable default as it does not introduce
    /// any extra attack vectors that the creator might be forced to consider
    /// and dynamic features can always be added at a later date.
    fn default() -> Self {
        Self(false)
    }
}

impl Display for Dynamic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = match self.0 {
            true => "Dynamic",
            false => "Static",
        };

        f.write_str(string)
    }
}

impl Dynamic {
    pub fn new(dynamic: bool) -> Self {
        Self(dynamic)
    }

    pub fn is_dynamic(&self) -> bool {
        self.0
    }

    pub fn write_move_defs(&self, type_name: &str) -> String {
        let mut code = String::new();

        if !self.0 {
            return code;
        }

        code.push_str(&write_move_field_setter(
            type_name,
            "name",
            &["name"],
            &["std::string::String"],
            "name",
        ));

        code.push_str(&write_move_field_setter(
            type_name,
            "description",
            &["description"],
            &["std::string::String"],
            "description",
        ));

        code.push_str(&write_move_field_setter(
            type_name,
            "url",
            &["url"],
            &["vector<u8>"],
            "sui::url::new_unsafe_from_bytes(url)",
        ));

        code.push_str(&write_move_field_setter(
            type_name,
            "attributes",
            &["attribute_keys", "attribute_values"],
            &["vector<std::ascii::String>", "vector<std::ascii::String>"],
            "nft_protocol::attributes::from_vec(attribute_keys, attribute_values)",
        ));

        code
    }

    pub fn write_move_tests(
        &self,
        type_name: &str,
        witness_name: &str,
        requires_collection: bool,
    ) -> String {
        let mut code = String::new();

        if !self.is_dynamic() {
            return code;
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

        code.push_str(&format!("

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

        let nft = mint(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,{collection_param_str}
            sui::test_scenario::ctx(&mut scenario)
        );
        let nft_id = sui::object::id(&nft);

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

        set_name_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            std::string::utf8(b\"Joystick\"),
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_description_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            std::string::utf8(b\"A test collection of Joysticks\"),
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_url_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            b\"https://docs.originbyte.io/\",
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_attributes_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            vector[std::ascii::string(b\"reveal\")],
            vector[std::ascii::string(b\"revealed\")],
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_to_address(CREATOR, publisher);
        sui::test_scenario::return_shared(borrow_policy);{collection_return_str}
        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::end(scenario);
    }}"));

        code
    }
}

fn write_move_field_setter(
    type_name: &str,
    field_name: &str,
    params: &[&str],
    param_types: &[&str],
    init_str: &str,
) -> String {
    let delegated_witness_param_type_str =
        format!("ob_permissions::witness::Witness<{type_name}>");
    let nft_param_type_str = format!("&mut {type_name}");
    let set_params_str = params.join(", ");

    let mut field_str = String::new();

    let mut set_params = Vec::new();
    set_params.extend_from_slice(&["_delegated_witness", "nft"]);
    set_params.extend_from_slice(params);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        delegated_witness_param_type_str.as_str(),
        nft_param_type_str.as_str(),
    ]);
    set_param_types.extend_from_slice(param_types);

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
    set_params.extend_from_slice(params);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        "&sui::package::Publisher",
        nft_param_type_str.as_str(),
    ]);
    set_param_types.extend_from_slice(param_types);

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
    set_params.extend_from_slice(params);
    set_params.extend_from_slice(&["policy", "ctx"]);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        delegated_witness_param_type_str.as_str(),
        "&mut sui::kiosk::Kiosk",
        "sui::object::ID",
    ]);
    set_param_types.extend_from_slice(param_types);
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
    set_params.extend_from_slice(params);
    set_params.extend_from_slice(&["policy", "ctx"]);

    let mut set_param_types = Vec::new();
    set_param_types.extend_from_slice(&[
        "&sui::package::Publisher",
        "&mut sui::kiosk::Kiosk",
        "sui::object::ID",
    ]);
    set_param_types.extend_from_slice(param_types);
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
