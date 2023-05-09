module gnomes::gnomes {
    /// One time witness is only instantiated in the init method
    struct GNOMES has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    struct Gnome has key, store {
                id: UID,
                name: String,
                description: String,
                url: Url,
                attributes: Attributes,
            }


    fun init(witness: GNOMES, ctx: &mut sui::tx_context::TxContext) {
        let sender = sui::tx_context::sender(ctx);

        let (collection, mint_cap) = nft_protocol::collection::create_with_mint_cap<GNOMES, Gnome>(
            &witness, option::none(), ctx
        );

        // Init Publisher
        let publisher = sui::package::claim(witness, ctx);

        // Init Tags
        let tags = nft_protocol::tags::empty(ctx);
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::art());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::profile_picture());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::collectible());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::game_asset());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::tokenised_asset());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::ticker());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::domain_name());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::music());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::video());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::ticket());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::license());
        nft_protocol::tags::add_tag(&mut tags, nft_protocol::tags::utility());


        // Init Display
        let display = sui::display::new<Gnome>(&publisher, ctx);
        sui::display::add(&mut display, std::string::utf8(b"name"), std::string::utf8(b"{name}"));
        sui::display::add(&mut display, std::string::utf8(b"description"), std::string::utf8(b"{description}"));
        sui::display::add(&mut display, std::string::utf8(b"image_url"), std::string::utf8(b"{url}"));
        sui::display::add(&mut display, std::string::utf8(b"attributes"), std::string::utf8(b"{attributes}"));
        sui::display::add(&mut display, std::string::utf8(b"tags"), ob_utils::display::from_vec(tags));
        sui::display::update_version(&mut display);

        let delegated_witness = nft_protocol::witness::from_witness(Witness {});

        let creators = sui::vec_set::empty();
        sui::vec_set::insert(&mut creators, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::vec_set::insert(&mut creators, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49dg);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::new(creators),
        );

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            display_info::new(
                std::string::utf8(b"gnomes"),
                std::string::utf8(b"Test contract generated by Gutenberg"),
            ),
        );

        let royalty_map = sui::vec_map::empty();
        sui::vec_map::insert(&mut royalty_map, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49dh, 1000);
        sui::vec_map::insert(&mut royalty_map, sui::tx_context::sender(ctx), 9000);

        nft_protocol::royalty_strategy_bps::create_domain_and_add_strategy(
            delegated_witness,
            &mut collection,
            nft_protocol::royalty::from_shares(royalty_map, ctx),
            500,
            ctx,
        );

        // Setup Kiosks for royalty address(es)
        ob_kiosk::ob_kiosk::init_for_address(@0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49dh, ctx);
        ob_kiosk::ob_kiosk::init_for_address(sui::tx_context::sender(ctx), ctx);

        let (transfer_policy, transfer_policy_cap) =
            ob_request::transfer_request::init_policy<Gnome>(&sui::package::publisher, ctx);

        nft_protocol::royalty_strategy_bps::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        nft_protocol::transfer_allowlist::enforce(
            &mut transfer_policy, &transfer_policy_cap,
        );
        let (borrow_policy, borrow_policy_cap) =
            ob_request::borrow_request::init_policy<Gnome>(&sui::package::publisher, ctx);

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<Gnome, SUI>(
            dw, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);
        // Setup Allowlist
        let (allowlist, allowlist_cap) = ob_allowlist::allowlist::new(ctx);

        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::orderbook::Witness>(
            &allowlist_cap, &mut allowlist,
        );
        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::bidding::Witness>(
            &allowlist_cap, &mut allowlist,
        );
        // Setup Transfers
        sui::transfer::public_transfer(publisher, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::transfer::public_transfer(mint_cap, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::transfer::public_transfer(allowlist_cap, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::transfer::public_share_object(allowlist);
        sui::transfer::public_share_object(collection);
        
        sui::transfer::public_transfer(transfer_policy_cap, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::transfer::public_share_object(transfer_policy);

        sui::transfer::public_transfer(borrow_policy_cap, @0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df);
        sui::transfer::public_share_object(borrow_policy);
    }

    public entry fun mint_nft(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
        warehouse: &mut nft_protocol::warehouse::Warehouse<Gnome>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            url,
            mint_cap,
            ctx,
        );

        nft_protocol::warehouse::deposit_nft(warehouse, nft);
    }

    public entry fun airdrop_nft(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Gnome>,
        receiver: &mut ob_kiosk::ob_kiosk::Kiosk,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            url,
            mint_cap,
            ctx,
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);
    }

    fun mint(
        name: std::string::String,
        url: vector<u8>,
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
        orderbook: &mut liquidity_layer_v1::Orderbook<Gnome, SUI>,
    ) {
        let dw = witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            dw, orderbook, liquidity_layer_v1::orderbook::custom_protection(false, false, false),
        );
    }

    public entry fun disable_orderbook(
        publisher: &sui::package::Publisher,
        orderbook: &mut liquidity_layer_v1::Orderbook<Gnome, SUI>,
    ) {
        let dw = witness::from_publisher(publisher);

        liquidity_layer_v1::orderbook::set_protection(
            dw, orderbook, liquidity_layer_v1::orderbook::custom_protection(true, true, true),
        );
    }

    #[test_only]
    use sui::test_scenario::{Self, ctx};
    #[test_only]
    use nft_protocol::collection::Collection;

    #[test_only]
    const CREATOR: address = @0xA1C04;

    #[test]
    fun it_inits_collection() {
        let scenario = test_scenario::begin(CREATOR);

        init(GNOMES {}, ctx(&mut scenario));
        test_scenario::next_tx(&mut scenario, CREATOR);

        assert!(test_scenario::has_most_recent_shared<Collection<Gnome>>(), 0);

        let mint_cap = test_scenario::take_from_address<MintCap<Gnome>>(
            &scenario, CREATOR,
        );

        test_scenario::return_to_address(CREATOR, mint_cap);
        test_scenario::next_tx(&mut scenario, CREATOR);

        test_scenario::end(scenario);
    }

    #[test]
    fun it_mints_nft() {
        let scenario = test_scenario::begin(CREATOR);
        init(GNOMES {}, ctx(&mut scenario));

        test_scenario::next_tx(&mut scenario, CREATOR);

        let mint_cap = test_scenario::take_from_address<MintCap<Gnome>>(
            &scenario,
            CREATOR,
        );

        let warehouse = warehouse::new<Gnome>(ctx(&mut scenario));

        mint_nft(
            string::utf8(b"TEST NAME"),
            string::utf8(b"TEST DESCRIPTION"),
            b"https://originbyte.io/",
            vector[ascii::string(b"avg_return")],
            vector[ascii::string(b"24%")],
            &mut mint_cap,
            &mut warehouse,
            ctx(&mut scenario)
        );

        transfer::public_transfer(warehouse, CREATOR);
        test_scenario::return_to_address(CREATOR, mint_cap);
        test_scenario::end(scenario);
    }
}
