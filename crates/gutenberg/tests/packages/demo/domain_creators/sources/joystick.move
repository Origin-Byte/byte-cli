module domain_creators::joystick {
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

        let creators = sui::vec_set::empty();
        sui::vec_set::insert(&mut creators, @0x61028a4c388514000a7de787c3f7b8ec1eb88d1bd2dbc0d3dfab37078e39630f);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::new(creators),
        );

        sui::transfer::public_share_object(collection);

        let mint_cap = nft_protocol::mint_cap::new_limited<JOYSTICK, Joystick>(
            &witness, collection_id, 100, ctx
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

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));
    }

    public entry fun mint_nft_to_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attributes_keys: vector<std::ascii::String>,
        attributes_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        receiver: &mut sui::kiosk::Kiosk,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            description,
            url,
            attributes_keys,
            attributes_values,
            mint_cap,
            ctx,
        );

        ob_kiosk::ob_kiosk::deposit(receiver, nft, ctx);
    }

    public entry fun mint_nft_to_new_kiosk(
        name: std::string::String,
        description: std::string::String,
        url: vector<u8>,
        attributes_keys: vector<std::ascii::String>,
        attributes_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        receiver: address,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let nft = mint(
            name,
            description,
            url,
            attributes_keys,
            attributes_values,
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
        attributes_keys: vector<std::ascii::String>,
        attributes_values: vector<std::ascii::String>,
        mint_cap: &mut nft_protocol::mint_cap::MintCap<Joystick>,
        ctx: &mut sui::tx_context::TxContext,
    ): Joystick {
        let delegated_witness = ob_permissions::witness::from_witness(Witness {});

        let nft = Joystick {
            id: sui::object::new(ctx),
            name,
            description,
            url: sui::url::new_unsafe_from_bytes(url),
            attributes: nft_protocol::attributes::from_vec(attributes_keys, attributes_values),
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
            std::string::utf8(b"TEST STRING"),
            std::string::utf8(b"TEST STRING"),
            b"https://originbyte.io",
            vector[std::ascii::string(b"key")],
            vector[std::ascii::string(b"attribute")],
            &mut mint_cap,
            &mut kiosk,
            sui::test_scenario::ctx(&mut scenario)
        );

        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::end(scenario);
    }
}
