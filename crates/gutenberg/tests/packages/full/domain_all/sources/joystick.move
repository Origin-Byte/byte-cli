module domain_all::joystick {
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

        let creators = sui::vec_set::empty();
        sui::vec_set::insert(&mut creators, @0x61028a4c388514000a7de787c3f7b8ec1eb88d1bd2dbc0d3dfab37078e39630f);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::new(creators),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::display_info::new(
                std::string::utf8(b"Joysticks"),
                std::string::utf8(b"Joysticks is a dummy collection"),
            ),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::symbol::new(std::string::utf8(b"JOYS")),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::supply::new(
                delegated_witness, 600, false,
            )
        );

        let royalty_map = sui::vec_map::empty();
        sui::vec_map::insert(&mut royalty_map, @0x61028a4c388514000a7de787c3f7b8ec1eb88d1bd2dbc0d3dfab37078e39630f, 500);
        sui::vec_map::insert(&mut royalty_map, @0x8212bb78cc5c42f95766107573147d79b0954fe06e52f54f27e26677b43c88f5, 9500);

        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            700,
            ctx,
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

        let tags = std::vector::empty();
        std::vector::push_back(&mut tags, nft_protocol::tags::art());
        std::vector::push_back(&mut tags, nft_protocol::tags::profile_picture());
        std::vector::push_back(&mut tags, nft_protocol::tags::collectible());
        std::vector::push_back(&mut tags, nft_protocol::tags::game_asset());
        std::vector::push_back(&mut tags, nft_protocol::tags::tokenised_asset());
        std::vector::push_back(&mut tags, nft_protocol::tags::domain_name());
        std::vector::push_back(&mut tags, nft_protocol::tags::music());
        std::vector::push_back(&mut tags, nft_protocol::tags::video());
        std::vector::push_back(&mut tags, nft_protocol::tags::ticket());
        std::vector::push_back(&mut tags, nft_protocol::tags::license());
        std::vector::push_back(&mut tags, std::string::utf8(b"Custom"));
        std::vector::push_back(&mut tags, std::string::utf8(b"Gaming"));
        std::vector::push_back(&mut tags, std::string::utf8(b"Utility"));

        sui::display::add(&mut display, std::string::utf8(b"tags"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);

        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
    }

    public entry fun mint_nft_to_warehouse(
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        collection: &mut nft_protocol::collection::Collection<Joystick>,
        warehouse: &mut ob_launchpad::warehouse::Warehouse<Joystick>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            mint_cap,
            collection,
            ctx,
        );

        ob_launchpad::warehouse::deposit_nft(warehouse, nft);
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
    fun it_mints_nft_launchpad() {
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

        let warehouse = ob_launchpad::warehouse::new<Joystick>(sui::test_scenario::ctx(&mut scenario));

        mint_nft_to_warehouse(
            &mut mint_cap,
            &mut collection,
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_shared(collection);
        sui::test_scenario::end(scenario);
    }
}
