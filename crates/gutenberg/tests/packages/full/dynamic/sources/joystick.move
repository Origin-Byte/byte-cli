module dynamic::joystick {
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

        let (borrow_policy, borrow_policy_cap) = ob_request::borrow_request::init_policy<Joystick>(
            &publisher, ctx,
        );

        sui::transfer::public_transfer(publisher, sui::tx_context::sender(ctx));

        sui::transfer::public_transfer(borrow_policy_cap, sui::tx_context::sender(ctx));
        sui::transfer::public_share_object(borrow_policy);
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

    public fun set_name(
        _delegated_witness: ob_permissions::witness::Witness<Joystick>,
        nft: &mut Joystick,
        name: std::string::String,
    ) {
        nft.name = name;
    }

    public entry fun set_name_as_publisher(
        publisher: &sui::package::Publisher,
        nft: &mut Joystick,
        name: std::string::String,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_name(delegated_witness, nft, name);
    }

    public fun set_name_in_kiosk(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        name: std::string::String,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<Joystick>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut Joystick = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_name(delegated_witness, nft, name);

        ob_kiosk::ob_kiosk::return_nft<Witness, Joystick>(kiosk, borrow, policy);
    }

    public entry fun set_name_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        name: std::string::String,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_name_in_kiosk(delegated_witness, kiosk, nft_id, name, policy, ctx);
    }

    public fun set_description(
        _delegated_witness: ob_permissions::witness::Witness<Joystick>,
        nft: &mut Joystick,
        description: std::string::String,
    ) {
        nft.description = description;
    }

    public entry fun set_description_as_publisher(
        publisher: &sui::package::Publisher,
        nft: &mut Joystick,
        description: std::string::String,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_description(delegated_witness, nft, description);
    }

    public fun set_description_in_kiosk(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        description: std::string::String,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<Joystick>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut Joystick = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_description(delegated_witness, nft, description);

        ob_kiosk::ob_kiosk::return_nft<Witness, Joystick>(kiosk, borrow, policy);
    }

    public entry fun set_description_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        description: std::string::String,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_description_in_kiosk(delegated_witness, kiosk, nft_id, description, policy, ctx);
    }

    public fun set_url(
        _delegated_witness: ob_permissions::witness::Witness<Joystick>,
        nft: &mut Joystick,
        url: vector<u8>,
    ) {
        nft.url = sui::url::new_unsafe_from_bytes(url);
    }

    public entry fun set_url_as_publisher(
        publisher: &sui::package::Publisher,
        nft: &mut Joystick,
        url: vector<u8>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_url(delegated_witness, nft, url);
    }

    public fun set_url_in_kiosk(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        url: vector<u8>,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<Joystick>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut Joystick = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_url(delegated_witness, nft, url);

        ob_kiosk::ob_kiosk::return_nft<Witness, Joystick>(kiosk, borrow, policy);
    }

    public entry fun set_url_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        url: vector<u8>,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_url_in_kiosk(delegated_witness, kiosk, nft_id, url, policy, ctx);
    }

    public fun set_attributes(
        _delegated_witness: ob_permissions::witness::Witness<Joystick>,
        nft: &mut Joystick,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
    ) {
        nft.attributes = nft_protocol::attributes::from_vec(attribute_keys, attribute_values);
    }

    public entry fun set_attributes_as_publisher(
        publisher: &sui::package::Publisher,
        nft: &mut Joystick,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_attributes(delegated_witness, nft, attribute_keys, attribute_values);
    }

    public fun set_attributes_in_kiosk(
        delegated_witness: ob_permissions::witness::Witness<Joystick>,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let borrow = ob_kiosk::ob_kiosk::borrow_nft_mut<Joystick>(kiosk, nft_id, std::option::none(), ctx);

        let nft: &mut Joystick = ob_request::borrow_request::borrow_nft_ref_mut(delegated_witness, &mut borrow);
        set_attributes(delegated_witness, nft, attribute_keys, attribute_values);

        ob_kiosk::ob_kiosk::return_nft<Witness, Joystick>(kiosk, borrow, policy);
    }

    public entry fun set_attributes_in_kiosk_as_publisher(
        publisher: &sui::package::Publisher,
        kiosk: &mut sui::kiosk::Kiosk,
        nft_id: sui::object::ID,
        attribute_keys: vector<std::ascii::String>,
        attribute_values: vector<std::ascii::String>,
        policy: &ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let delegated_witness = ob_permissions::witness::from_publisher(publisher);
        set_attributes_in_kiosk(delegated_witness, kiosk, nft_id, attribute_keys, attribute_values, policy, ctx);
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
    fun it_sets_metadata() {
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

        let borrow_policy: ob_request::request::Policy<ob_request::request::WithNft<Joystick, ob_request::borrow_request::BORROW_REQ>> =
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

        set_name_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            std::string::utf8(b"Joystick"),
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_description_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            std::string::utf8(b"A test collection of Joysticks"),
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_url_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            b"https://docs.originbyte.io/",
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        set_attributes_in_kiosk_as_publisher(
            &publisher,
            &mut kiosk,
            nft_id,
            vector[std::ascii::string(b"reveal")],
            vector[std::ascii::string(b"revealed")],
            &borrow_policy,
            sui::test_scenario::ctx(&mut scenario),
        );

        sui::test_scenario::return_to_address(CREATOR, mint_cap);
        sui::test_scenario::return_to_address(CREATOR, publisher);
        sui::test_scenario::return_shared(borrow_policy);
        sui::transfer::public_share_object(kiosk);
        sui::test_scenario::end(scenario);
    }
}
