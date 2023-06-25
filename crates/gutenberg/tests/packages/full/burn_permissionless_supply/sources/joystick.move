module burn_permissionless_supply::joystick {
    /// One time witness is only instantiated in the init method
    struct JOYSTICK has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    struct Joystick has key, store {
        id: sui::object::UID,
    }

    fun init(witness: JOYSTICK, ctx: &mut sui::tx_context::TxContext) {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let collection = nft_protocol::collection::create<Joystick>(delegated_witness, ctx);
        let collection_id = sui::object::id(&collection);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::supply::new(
                delegated_witness, 600, false,
            )
        );

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
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        receiver: &mut sui::kiosk::Kiosk,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            mint_cap,
            collection,
            ctx,
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);
    }

    public entry fun mint_nft_to_new_kiosk(
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        receiver: address,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            mint_cap,
            collection,
            ctx,
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new_for_address(receiver, ctx);
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, ctx);
        sui::transfer::public_share_object(kiosk);
    }

    fun mint(
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        ctx: &mut sui::tx_context::TxContext,
    ): Joystick {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let nft = Joystick {
            id: sui::object::new(ctx),
        };

        nft_protocol::mint_event::emit_mint(
            delegated_witness,
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );

        let supply = nft_protocol::supply::borrow_domain_mut(
            nft_protocol::collection::borrow_uid_mut(delegated_witness, collection),
        );

        nft_protocol::supply::increment(delegated_witness, supply, 1);

        nft_protocol::mint_cap::increment_supply(mint_cap, 1);

        nft
    }

    public fun burn_nft(
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        nft: Joystick,
    ) {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);
        let Joystick { id } = nft;
        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);

        let supply = nft_protocol::supply::borrow_domain_mut(
            nft_protocol::collection::borrow_uid_mut(delegated_witness, collection),
        );

        nft_protocol::supply::decrement(delegated_witness, supply, 1);
        nft_protocol::supply::decrease_supply_ceil(delegated_witness, supply, 1);
    }

    public entry fun burn_nft_in_kiosk(
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);
        ob_request::withdraw_request::add_receipt(&mut withdraw_request, &Witness {});
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_nft(collection, nft);
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

        let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<Joystick>>(
            &scenario,
        );

        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_kiosk(
            &mut mint_cap,
            &mut collection,
            &mut kiosk,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_shared(collection);
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
                &mut mint_cap,
            &mut collection,
                sui::test_scenario::ctx(&mut scenario)
            );
            let nft_id = sui::object::id(&nft);

            let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
            ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));

            burn_nft_in_kiosk(
                &mut collection,
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
