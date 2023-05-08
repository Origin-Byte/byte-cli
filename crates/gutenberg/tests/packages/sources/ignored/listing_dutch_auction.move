module gutenberg::listingdutchauction {
    /// One time witness is only instantiated in the init method
    struct LISTINGDUTCHAUCTION has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    fun init(witness: LISTINGDUTCHAUCTION, ctx: &mut sui::tx_context::TxContext) {
        let (mint_cap, collection) = nft_protocol::collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness<LISTINGDUTCHAUCTION, Witness>(&Witness {});

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::from_address<LISTINGDUTCHAUCTION, Witness>(
                &Witness {}, sui::tx_context::sender(ctx),
            ),
        );

        sui::transfer::transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::share_object(collection);
    }

    public entry fun mint_to_launchpad(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &nft_protocol::mint_cap::MintCap<LISTINGDUTCHAUCTION>,
        warehouse: &mut nft_protocol::warehouse::Warehouse<LISTINGDUTCHAUCTION>,
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

    fun mint(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &nft_protocol::mint_cap::MintCap<LISTINGDUTCHAUCTION>,
        ctx: &mut sui::tx_context::TxContext,
    ): nft_protocol::nft::Nft<LISTINGDUTCHAUCTION> {
        let nft = nft_protocol::nft::from_mint_cap(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );
        let delegated_witness = nft_protocol::witness::from_witness<LISTINGDUTCHAUCTION, Witness>(&Witness {});

        nft
    }
}
