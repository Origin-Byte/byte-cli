module orderbook_protected_royalty::joystick {
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

        let royalty_map = sui::vec_map::empty();

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
        sui::display::update_version(&mut display);

        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));

        let (transfer_policy, transfer_policy_cap) = ob_request::transfer_request::init_policy<Joystick>(
            &publisher, ctx,
        );
        nft_protocol::royalty_strategy_bps::enforce(&mut transfer_policy, &transfer_policy_cap);
        nft_protocol::transfer_allowlist::enforce(&mut transfer_policy, &transfer_policy_cap);

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<Joystick, sui::sui::SUI>(
            delegated_witness, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);
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
    fun test_trade() {
        let scenario = sui::test_scenario::begin(CREATOR);

        init(JOYSTICK {}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        // Setup allowlist
        let (allowlist, allowlist_cap) = ob_allowlist::allowlist::new(sui::test_scenario::ctx(&mut scenario));
        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::orderbook::Witness>(&allowlist_cap, &mut allowlist);

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario, CREATOR,
        );

        // Need to insert all tradeable types into collection
        ob_allowlist::allowlist::insert_collection<Joystick>(&mut allowlist, &publisher);
        sui::transfer::public_share_object(allowlist);
        sui::transfer::public_transfer(allowlist_cap, CREATOR);

        // Setup orderbook
        let orderbook = sui::test_scenario::take_shared<liquidity_layer_v1::orderbook::Orderbook<Joystick, sui::sui::SUI>>(&scenario);
        liquidity_layer_v1::orderbook::enable_orderbook(&publisher, &mut orderbook);

        sui::test_scenario::return_to_address(CREATOR, publisher);

        // Setup test NFT
        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Joystick>>(
            &mut scenario, CREATOR,
        );

        let nft = mint(
            std::string::utf8(b"TEST NAME"),
            std::string::utf8(b"TEST DESCRIPTION"),
            b"https://originbyte.io/",
            vector[std::ascii::string(b"avg_return")],
            vector[std::ascii::string(b"24%")],
            &mut mint_cap,
            sui::test_scenario::ctx(&mut scenario),
        );
        let nft_id = sui::object::id(&nft);

        sui::test_scenario::return_to_address(CREATOR, mint_cap);

        // Deposit NFT into Kiosk
        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));
        sui::transfer::public_share_object(kiosk);

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        // Test trade
        let seller_kiosk = sui::test_scenario::take_shared<sui::kiosk::Kiosk>(&scenario);
        let (buyer_kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));

        liquidity_layer_v1::orderbook::create_ask(
            &mut orderbook,
            &mut seller_kiosk,
            100_000_000,
            nft_id,
            sui::test_scenario::ctx(&mut scenario),
        );

        let coin = sui::coin::mint_for_testing<sui::sui::SUI>(100_000_000, sui::test_scenario::ctx(&mut scenario));

        let trade_opt = liquidity_layer_v1::orderbook::create_bid(
            &mut orderbook,
            &mut buyer_kiosk,
            100_000_000,
            &mut coin,
            sui::test_scenario::ctx(&mut scenario),
        );

        sui::coin::burn_for_testing(coin);
        let trade = std::option::destroy_some(trade_opt);

        let request = liquidity_layer_v1::orderbook::finish_trade(
            &mut orderbook,
            liquidity_layer_v1::orderbook::trade_id(&trade),
            &mut seller_kiosk,
            &mut buyer_kiosk,
            sui::test_scenario::ctx(&mut scenario),
        );

        let allowlist = sui::test_scenario::take_shared<ob_allowlist::allowlist::Allowlist>(&scenario);
        nft_protocol::transfer_allowlist::confirm_transfer(&allowlist, &mut request);
        sui::test_scenario::return_shared(allowlist);

        let royalty_strategy = sui::test_scenario::take_shared<nft_protocol::royalty_strategy_bps::BpsRoyaltyStrategy<Joystick>>(&mut scenario);
        nft_protocol::royalty_strategy_bps::confirm_transfer<Joystick, sui::sui::SUI>(&mut royalty_strategy, &mut request);
        sui::test_scenario::return_shared(royalty_strategy);

        let transfer_policy = sui::test_scenario::take_shared<sui::transfer_policy::TransferPolicy<Joystick>>(&scenario);
        ob_request::transfer_request::confirm<Joystick, sui::sui::SUI>(request, &transfer_policy, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::return_shared(transfer_policy);

        ob_kiosk::ob_kiosk::assert_nft_type<Joystick>(&buyer_kiosk, nft_id);

        sui::transfer::public_share_object(buyer_kiosk);
        sui::test_scenario::return_shared(seller_kiosk);
        sui::test_scenario::return_shared(orderbook);
        sui::test_scenario::end(scenario);
    }
}
