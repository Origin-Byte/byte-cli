module gutenberg::domaincreatorssender {
    /// One time witness is only instantiated in the init method
    struct DOMAINCREATORSSENDER has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    fun init(witness: DOMAINCREATORSSENDER, ctx: &mut sui::tx_context::TxContext) {
        let (mint_cap, collection) = nft_protocol::collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness<DOMAINCREATORSSENDER, Witness>(&Witness {});

        let creators = sui::vec_set::empty();
        sui::vec_set::insert(&mut creators, sui::tx_context::sender(ctx));
        sui::vec_set::insert(&mut creators, @0x64be9c21161c2305543e2bba67e056ebba8729e4);

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::from_creators<DOMAINCREATORSSENDER, Witness>(
                &Witness {}, creators,
            ),
        );

        sui::transfer::transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::share_object(collection);
    }

    fun mint(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &nft_protocol::mint_cap::MintCap<DOMAINCREATORSSENDER>,
        ctx: &mut sui::tx_context::TxContext,
    ): nft_protocol::nft::Nft<DOMAINCREATORSSENDER> {
        let nft = nft_protocol::nft::from_mint_cap(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );
        let delegated_witness = nft_protocol::witness::from_witness<DOMAINCREATORSSENDER, Witness>(&Witness {});

        nft
    }
}
