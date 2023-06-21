module burn_permissioned::joystick {
    /// One time witness is only instantiated in the init method
    struct JOYSTICK has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    struct Joystick has key, store {
        id: sui::object::UID,
        name: std::string::String,
        description: std::string::String,
        url: sui::url::Url,
        attributes: nft_protocol::attributes::Attributes,
    }

    fun init(witness: JOYSTICK, ctx: &mut sui::tx_context::TxContext) {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let collection = nft_protocol::collection::create<Joystick>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);

        sui::transfer::public_share_object(collection);

        let mint_cap = nft_protocol::mint_cap::new_unlimited<JOYSTICK, Joystick>(
            &witness, collection_id, ctx
        );
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));

        let publisher = sui::package::claim(witness, ctx);

        let display = sui::display::new<Joystick>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b"name"), std::string::utf8(b"{name}"));
        sui::display::add(&mut display, std::string::utf8(b"description"), std::string::utf8(b"{description}"));
        sui::display::add(&mut display, std::string::utf8(b"image_url"), std::string::utf8(b"{url}"));
        sui::display::add(&mut display, std::string::utf8(b"attributes"), std::string::utf8(b"{attributes}"));
        sui::display::update_version(&mut display);

        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));

        let (withdraw_policy, withdraw_policy_cap) = ob_request::withdraw_request::init_policy<Joystick>(
            &publisher, ctx,
        );
        ob_request::request::enforce_rule_no_state<ob_request::request::WithNft<Joystick, ob_request::withdraw_request::WITHDRAW_REQ>, Witness>(
            &mut withdraw_policy, &withdraw_policy_cap,
        );

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));

        sui::transfer::public_transfer(withdraw_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(withdraw_policy);
    }

    public entry fun mint_nft_to_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        receiver: &mut sui::kiosk::Kiosk,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            description,
            url,
            attribute_keys,
            attribute_values,
            mint_cap,
            ctx,
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);
    }

    public entry fun mint_nft_to_new_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        receiver: address,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            description,
            url,
            attribute_keys,
            attribute_values,
            mint_cap,
            ctx,
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new_for_address(receiver, ctx);
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, ctx);
        sui::transfer::public_share_object(kiosk);
    }

    fun mint(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        ctx: &mut sui::tx_context::TxContext,
    ): Joystick {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let nft = Joystick {
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attribute_keys, attribute_values)
        };

        nft_protocol::mint_event::emit_mint(
            delegated_witness,
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );

        nft_protocol::mint_cap::increment_supply(mint_cap, 1);

        nft
    }

    public fun burn_nft(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        collection: &nft_protocol::collection::Collection<Joystick>,
        nft: Joystick,
    ) {
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);
        let Joystick { id, name: _, description: _, url: _, attributes: _ } = nft;
        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);
    }

    public entry fun burn_nft_in_kiosk(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        collection: &nft_protocol::collection::Collection<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);
        ob_request::withdraw_request::add_receipt(&mut withdraw_request, &Witness {});
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_nft(delegated_witness, collection, nft);
    }

    public entry fun burn_nft_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        burn_nft_in_kiosk(delegated_witness, collection, kiosk, nft_id, policy, ctx);
    }

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {
        let scenario = sui::test_scenario::begin(CREATOR);

        init(JOYSTICK {}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(sui::test_scenario::has_most_recent_shared<nft_protocol::collection::Collection<Joystick>>(), 0);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Joystick>>(
            &scenario, CREATOR,
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        sui::test_scenario::end(scenario);
    }

    #[test]
    fun it_mints_nft_airdrop() {
        let scenario = sui::test_scenario::begin(CREATOR);
        init(JOYSTICK {}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Joystick>>(
            &scenario,
            CREATOR,
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_kiosk(
            std::string::utf8(b"TEST NAME"),
            std::string::utf8(b"TEST DESCRIPTION"),
            b"https://originbyte.io/",
            vector[std::ascii::string(b"avg_return")],
            vector[std::ascii::string(b"24%")],
            &mut mint_cap,
            &mut kiosk,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::end(scenario);
    }

        #[test]
        fun it_burns_nft() {
            let scenario = sui::test_scenario::begin(CREATOR);
            init(JOYSTICK {}, sui::test_scenario::ctx(&mut scenario));

            sui::test_scenario::next_tx(&mut scenario, CREATOR);

            let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Joystick>>(
                &scenario,
                CREATOR,
            );

            let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
                &scenario,
                CREATOR,
            );

            let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<Joystick>>(
                &scenario
            );

            let withdraw_policy = sui::test_scenario::take_shared<
                ob_request::request::Policy<
                    ob_request::request::WithNft<Joystick, ob_request::withdraw_request::WITHDRAW_REQ>
                >
            >(&scenario);

            let nft = mint(
                std::string::utf8(b"TEST NAME"),
                std::string::utf8(b"TEST DESCRIPTION"),
                b"https://originbyte.io/",
                vector[std::ascii::string(b"avg_return")],
                vector[std::ascii::string(b"24%")],
                &mut mint_cap,
                sui::test_scenario::ctx(&mut scenario)
            );
            let nft_id = sui::object::id(&nft);

            let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
            ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

            burn_nft_in_kiosk(
                ob_permissions::witness::from_witness(Witness {}),
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
        }
}
