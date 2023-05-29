module project_eluune_aurahma_pre_reveal::project_eluune_aurahma_pre_reveal {
    /// One time witness is only instantiated in the init method
    struct PROJECT_ELUUNE_AURAHMA_PRE_REVEAL has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    struct AurahmaPreReveal has key, store {
        id: sui::object::UID,
        name: std::string::String,
        description: std::string::String,
        url: sui::url::Url,
        attributes: nft_protocol::attributes::Attributes,
    }

    fun init(witness: PROJECT_ELUUNE_AURAHMA_PRE_REVEAL, ctx: &mut sui::tx_context::TxContext) {
        let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<PROJECT_ELUUNE_AURAHMA_PRE_REVEAL, AurahmaPreReveal>(
            &witness, std::option::some(600), ctx
        );

        // Init Publisher
        let publisher = sui::package::claim(witness, ctx);

        let tags: vector<std::string::String> = std::vector::empty();
        std::vector::push_back(&mut tags, std::string::utf8(b"Gaming"));

        // Init Display
        let display = sui::display::new<AurahmaPreReveal>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b"name"), std::string::utf8(b"{name}"));
        sui::display::add(&mut display, std::string::utf8(b"description"), std::string::utf8(b"{description}"));
        sui::display::add(&mut display, std::string::utf8(b"image_url"), std::string::utf8(b"{url}"));
        sui::display::add(&mut display, std::string::utf8(b"attributes"), std::string::utf8(b"{attributes}"));
        sui::display::add(&mut display, std::string::utf8(b"tags"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);
        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));

        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

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
                std::string::utf8(b"Project Eluune: Aurahma Pre-Reveal"),
                std::string::utf8(b"Project Eluune is a gaming sci-fi fantasy universe brought to you by OG game veterans who built Call Of Duty, Dota2 and many of your favorite video games. As a player, you answer the call of Eluune, a mysterious entity who has sent an SOS message to us humans from a hidden world... Techno-magical, teeming with fantastic life and rich secrecy spanning millions of years... Will you answer the call?"),
            ),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::symbol::new(std::string::utf8(b"PRAMA")),
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

        let (transfer_policy, transfer_policy_cap) =
            ob_request::transfer_request::init_policy<AurahmaPreReveal>(&publisher, ctx);

        nft_protocol::royalty_strategy_bps::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        nft_protocol::transfer_allowlist::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<AurahmaPreReveal, sui::sui::SUI>(
            delegated_witness, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);

        // Setup Transfers
        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(collection);

        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);
    }

    public entry fun mint_nft(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<AurahmaPreReveal>,
        warehouse: &mut ob_launchpad::warehouse::Warehouse<AurahmaPreReveal>,
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

        ob_launchpad::warehouse::deposit_nft(warehouse, nft);
    }


    public entry fun airdrop_nft(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<AurahmaPreReveal>,
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

    public entry fun airdrop_nft_into_new_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<AurahmaPreReveal>,
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
        mint_cap: &mut nft_protocol::mint_cap::MintCap<AurahmaPreReveal>,
        ctx: &mut sui::tx_context::TxContext,
    ): AurahmaPreReveal {
        let nft = AurahmaPreReveal {
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attribute_keys, attribute_values)
        };
        nft_protocol::mint_event::emit_mint(
            ob_permissions::witness::from_witness(Witness {}),
            nft_protocol::mint_cap::collection_id(mint_cap),
            &nft,
        );
        nft_protocol::mint_cap::increment_supply(mint_cap, 1);
        nft
    }

    // Protected orderbook functions
    public entry fun enable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<AurahmaPreReveal, sui::sui::SUI>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(false, false, false),
        );
    }

    public entry fun disable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<AurahmaPreReveal, sui::sui::SUI>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(true, true, true),
        );
    }

    public fun set_metadata(
        _delegated_witness: ob_permissions::witness::Witness<AurahmaPreReveal>,
        nft: &mut AurahmaPreReveal,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
    ) {
        nft.url = sui::url::new_unsafe_from_bytes(url);
        nft.attributes = nft_protocol::attributes::from_vec(attribute_keys, attribute_values);
    }

    public entry fun set_metadata_as_publisher(
        publisher: &sui::package::Publisher,
        nft: &mut AurahmaPreReveal,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_metadata(delegated_witness, nft, url, attribute_keys, attribute_values);
    }

    public entry fun set_metadata_in_kiosk(
        publisher: &sui::package::Publisher,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<AurahmaPreReveal, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<AurahmaPreReveal>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut AurahmaPreReveal = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_metadata(delegated_witness, nft, url, attribute_keys, attribute_values);

        ob_kiosk::ob_kiosk::return_nft<Witness, AurahmaPreReveal>(kiosk, borrow, policy);
    }

    public fun burn_nft(
        delegated_witness: ob_permissions::witness::Witness<AurahmaPreReveal>,
        collection: &nft_protocol::collection::Collection<AurahmaPreReveal>,
        nft: AurahmaPreReveal,
    ) {
        let guard = nft_protocol::mint_event::start_burn(delegated_witness, &nft);

        let AurahmaPreReveal { id, name: _, description: _, url: _, attributes: _ } = nft;

        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);
    }

    public entry fun burn_nft_in_listing(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<AurahmaPreReveal>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft(listing, inventory_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }

    public entry fun burn_nft_in_listing_with_id(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<AurahmaPreReveal>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        nft_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        let nft = ob_launchpad::listing::admin_redeem_nft_with_id(listing, inventory_id, nft_id, ctx);
        burn_nft(delegated_witness, collection, nft);
    }

    public entry fun burn_own_nft(
        collection: &nft_protocol::collection::Collection<AurahmaPreReveal>,
        nft: AurahmaPreReveal,
    ) {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});
        burn_nft(delegated_witness, collection, nft);
    }

    public entry fun burn_own_nft_in_kiosk(
        collection: &nft_protocol::collection::Collection<AurahmaPreReveal>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<AurahmaPreReveal, ob_request::withdraw_request::WITHDRAW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let (nft, withdraw_request) = ob_kiosk::ob_kiosk::withdraw_nft_signed(kiosk, nft_id, ctx);
        ob_request::withdraw_request::confirm(withdraw_request, policy);

        burn_own_nft(collection, nft);
    }

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {
        let scenario = sui::test_scenario::begin(CREATOR);

        init(PROJECT_ELUUNE_AURAHMA_PRE_REVEAL {}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(sui::test_scenario::has_most_recent_shared<nft_protocol::collection::Collection<AurahmaPreReveal>>(), 0);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<AurahmaPreReveal>>(
            &scenario, CREATOR,
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        sui::test_scenario::end(scenario);
    }

    #[test]
    fun it_mints_nft() {
        let scenario = sui::test_scenario::begin(CREATOR);
        init(PROJECT_ELUUNE_AURAHMA_PRE_REVEAL {}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<AurahmaPreReveal>>(
            &scenario,
            CREATOR,
        );

        let warehouse = ob_launchpad::warehouse::new<AurahmaPreReveal>(sui::test_scenario::ctx(&mut scenario));

        mint_nft(
            std::string::utf8(b"TEST NAME"),
            std::string::utf8(b"TEST DESCRIPTION"),
            b"https://originbyte.io/",
            vector[std::ascii::string(b"avg_return")],
            vector[std::ascii::string(b"24%")],
            &mut mint_cap,
            &mut warehouse,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_transfer(warehouse, CREATOR);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::end(scenario);
    }

    #[test]
    fun it_sets_metadata() {
        let scenario = sui::test_scenario::begin(CREATOR);
        init(PROJECT_ELUUNE_AURAHMA_PRE_REVEAL {}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<AurahmaPreReveal>>(
            &scenario,
            CREATOR,
        );

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario,
            CREATOR,
        );

        let borrow_policy: ob_request::request::Policy<ob_request::request::WithNft<AurahmaPreReveal, ob_request::borrow_request::BORROW_REQ>> =
            sui::test_scenario::take_shared(&mut scenario);

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

        set_metadata_in_kiosk(
            &publisher,
            &mut kiosk,
            nft_id,
            b"https://docs.originbyte.io/",
            vector[std::ascii::string(b"reveal")],
            vector[std::ascii::string(b"revealed")],
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_to_address(CREATOR, publisher);
        sui::test_scenario::return_shared(borrow_policy);
        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::end(scenario);
    }

    #[test]
    fun it_burns_own_nft() {
        let scenario = sui::test_scenario::begin(CREATOR);
        init(PROJECT_ELUUNE_AURAHMA_PRE_REVEAL {}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<AurahmaPreReveal>>(
            &scenario,
            CREATOR,
        );

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario,
            CREATOR,
        );

        let collection = sui::test_scenario::take_shared<
            nft_protocol::collection::Collection<AurahmaPreReveal>
        >(&scenario);

        let withdraw_policy = sui::test_scenario::take_shared<
            ob_request::request::Policy<
                ob_request::request::WithNft<AurahmaPreReveal, ob_request::withdraw_request::WITHDRAW_REQ>
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
    }
}
