module gnomes_inc::gnomes {
    /// One time witness is only instantiated in the init method
    struct GNOMES has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    struct Gnome has key, store {
        id: sui::object::UID,
        name: std::string::String,
        description: std::string::String,
        url: sui::url::Url,
        attributes: nft_protocol::attributes::Attributes,
    }

    fun init(witness: GNOMES, ctx: &mut sui::tx_context::TxContext) {
        let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<GNOMES, Gnome>(
            &witness, std::option::some(33333), ctx
        );

        // Init Publisher
        let publisher = sui::package::claim(witness, ctx);

        // Init Tags
        let tags: vector<std::string::String> = std::vector::empty();
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


        // Init Display
        let display = sui::display::new<Gnome>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b"name"), std::string::utf8(b"{name}"));
        sui::display::add(&mut display, std::string::utf8(b"description"), std::string::utf8(b"{description}"));
        sui::display::add(&mut display, std::string::utf8(b"image_url"), std::string::utf8(b"{url}"));
        sui::display::add(&mut display, std::string::utf8(b"attributes"), std::string::utf8(b"{attributes}"));
        sui::display::add(&mut display, std::string::utf8(b"tags"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);
        sui::transfer::public_transfer(display, sui::tx_context::sender(ctx));

        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let creators = sui::vec_set::empty();
        sui::vec_set::insert(&mut creators, @0x0b86be5d779fac217b41d484b8040ad5145dc9ba0cba099d083c6cbda50d983e);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::new(creators),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::display_info::new(
                std::string::utf8(b"gnomes"),
                std::string::utf8(b"Test contract generated by Gutenberg"),
            ),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::symbol::new(std::string::utf8(b"GNOMES")),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            sui::url::new_unsafe_from_bytes(b"https://originbyte.io/"),
        );

        let royalty_map = sui::vec_map::empty();
        sui::vec_map::insert(&mut royalty_map, @0x0b86be5d779fac217b41d484b8040ad5145dc9ba0cba099d083c6cbda50d983e, 1000);
        sui::vec_map::insert(&mut royalty_map, @sui::tx_context::sender(ctx), 9000);

        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            500,
            ctx,
        );


        let (transfer_policy, transfer_policy_cap) =
            ob_request::transfer_request::init_policy<Gnome>(&publisher, ctx);

        nft_protocol::royalty_strategy_bps::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        nft_protocol::transfer_allowlist::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        let (borrow_policy, borrow_policy_cap) =
            ob_request::borrow_request::init_policy<Gnome>(&publisher, ctx);

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<Gnome, sui::sui::SUI>(
            delegated_witness, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);
        // Setup Transfers
        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
        sui::transfer::public_transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(collection);
        
        sui::transfer::public_transfer(transfer_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(transfer_policy);

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);
    }

    public entry fun mint_nft(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
        warehouse: &mut ob_launchpad::warehouse::Warehouse<Gnome>,
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
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
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
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
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
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
        ctx: &mut sui::tx_context::TxContext,
    ): Gnome {
        
        let nft = Gnome {
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
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<Gnome, sui::sui::SUI>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(false, false, false),
        );
    }

    public entry fun disable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::orderbook::Orderbook<Gnome, sui::sui::SUI>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            delegated_witness, orderbook, liquidity_layer_v1::orderbook::custom_protection(true, true, true),
        );
    }
    // Burn functions
    public entry fun burn_nft(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<Gnome>,
        nft: Gnome,
    ) {
        let dw = ob_permissions::witness::from_publisher(publisher);
        let guard = nft_protocol::mint_event::start_burn(dw, &nft);

        let Gnome { id, name: _, description: _, url: _, attributes: _ } = nft;

        nft_protocol::mint_event::emit_burn(guard, sui::object::id(collection), id);
    }
        
    public entry fun burn_nft_in_listing(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<Gnome>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = ob_launchpad::listing::admin_redeem_nft(listing, inventory_id, ctx);
        burn_nft(publisher, collection, nft);
    }
        
    public entry fun burn_nft_in_listing_with_id(
        publisher: &sui::package::Publisher,
        collection: &nft_protocol::collection::Collection<Gnome>,
        listing: &mut ob_launchpad::listing::Listing,
        inventory_id: sui::object::ID,
        nft_id: sui::object::ID,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = ob_launchpad::listing::admin_redeem_nft_with_id(listing, inventory_id, nft_id, ctx);
        burn_nft(publisher, collection, nft);
    }
        

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {
        let scenario = sui::test_scenario::begin(CREATOR);

        init(GNOMES {}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(sui::test_scenario::has_most_recent_shared<nft_protocol::collection::Collection<Gnome>>(), 0);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Gnome>>(
            &scenario, CREATOR,
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        sui::test_scenario::end(scenario);
    }

    #[test]
    fun it_mints_nft() {
        let scenario = sui::test_scenario::begin(CREATOR);
        init(GNOMES {}, sui::test_scenario::ctx(&mut scenario));

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<Gnome>>(
            &scenario,
            CREATOR,
        );

        let warehouse = ob_launchpad::warehouse::new<Gnome>(sui::test_scenario::ctx(&mut scenario));

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
}
