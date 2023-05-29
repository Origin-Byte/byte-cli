use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Burn {
    None,
    Permissioned,
    Permissionless,
}

impl Burn {
    pub fn is_none(&self) -> bool {
        match self {
            Burn::None => true,
            _ => false,
        }
    }

    pub fn is_permissioned(&self) -> bool {
        match self {
            Burn::Permissioned => true,
            _ => false,
        }
    }

    pub fn is_permissionless(&self) -> bool {
        match self {
            Burn::Permissionless => true,
            _ => false,
        }
    }

    pub fn write_burn_fns(&self, nft_type_name: &String) -> String {
        let mut code = String::new();

        if let Burn::None = self {
            return code;
        }

        code.push_str(&format!(
            "

    public fun burn_nft(
        delegated_witness: ob_permissions::witness::Witness<{nft_type_name}>,
        collection: &nft_protocol::collection::Collection<{nft_type_name}>,
        nft: {nft_type_name},
    ) {{
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);

        let {nft_type_name} {{ id, name: _, description: _, url: _, attributes: _ }} = nft;

        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);
    }}"
        ));

        code.push_str(&format!(
            "

    public entry fun burn_nft_in_listing(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<{nft_type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft(listing, inventory_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }}"
        ));

        code.push_str(&format!(
            "

    public entry fun burn_nft_in_listing_with_id(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<{nft_type_name}>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        nft_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft_with_id(listing, inventory_id, nft_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }}"
        ));

        if self == &Burn::Permissionless {
            code.push_str(&format!(
            "

    public entry fun burn_own_nft(
        collection: &nft_protocol::collection::Collection<{nft_type_name}>,
        nft: {nft_type_name},
    ) {{
        let delegated_witness = ob_permissions::witness::from_witness(Witness {{}});
        burn_nft(delegated_witness, collection, nft);
    }}

    public entry fun burn_own_nft_in_kiosk(
        collection: &nft_protocol::collection::Collection<{nft_type_name}>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<{nft_type_name}, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {{
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_own_nft(collection, nft);
    }}"
        ));
        }

        code
    }

    pub fn write_burn_tests(
        &self,
        nft_type_name: &String,
        witness_type: &String,
    ) -> String {
        match self {
            Burn::None => String::new(),
            Burn::Permissioned => String::new(),
            Burn::Permissionless => {
                format!(
                "

    #[test]
    fun it_burns_own_nft() {{
        let scenario = sui::test_scenario::begin(CREATOR);
        init({witness_type} {{}}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{nft_type_name}>>(
            &scenario,
            CREATOR,
        );

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario,
            CREATOR,
        );

        let collection = sui::test_scenario::take_shared<
            nft_protocol::collection::Collection<{nft_type_name}>
        >(&scenario);

        let withdraw_policy = sui::test_scenario::take_shared<
            ob_request::request::Policy<
                ob_request::request::WithNft<{nft_type_name}, ob_request::withdraw_request::WITHDRAW_REQ>
            >
        >(&scenario);

        let nft = mint(
            std::string::utf8(b\"TEST NAME\"),
            std::string::utf8(b\"TEST DESCRIPTION\"),
            b\"https://originbyte.io/\",
            vector[std::ascii::string(b\"avg_return\")],
            vector[std::ascii::string(b\"24%\")],
            &mut mint_cap,
            sui::test_scenario::ctx(&mut scenario)
        );
        let nft_id = sui::object::id(&nft);

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

        burn_own_nft_in_kiosk(
            &collection,
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
    }
}
