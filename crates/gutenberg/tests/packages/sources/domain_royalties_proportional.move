module gutenberg::domainroyaltiesproportional {
    /// One time witness is only instantiated in the init method
    struct DOMAINROYALTIESPROPORTIONAL has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}

    fun init(witness: DOMAINROYALTIESPROPORTIONAL, ctx: &mut sui::tx_context::TxContext) {
        let (mint_cap, collection) = nft_protocol::collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness<DOMAINROYALTIESPROPORTIONAL, Witness>(&Witness {});

        nft_protocol::collection::add_domain(
            delegated_witness,
            &mut collection,
            nft_protocol::creators::from_address<DOMAINROYALTIESPROPORTIONAL, Witness>(
                &Witness {}, sui::tx_context::sender(ctx),
            ),
        );

        let royalty_map = sui::vec_map::empty();
        sui::vec_map::insert(&mut royalty_map, @0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d144, 5000);
        sui::vec_map::insert(&mut royalty_map, sui::tx_context::sender(ctx), 5000);

        let royalty = nft_protocol::royalty::from_shares(royalty_map, ctx);
        nft_protocol::royalty::add_proportional_royalty(&mut royalty, 100);
        nft_protocol::royalty::add_royalty_domain(delegated_witness, &mut collection, royalty);

        sui::transfer::transfer(mint_cap, sui::tx_context::sender(ctx));
        sui::transfer::share_object(collection);
    }

    /// Calculates and transfers royalties to the `RoyaltyDomain`
    public entry fun collect_royalty<FT>(
        payment: &mut nft_protocol::royalties::TradePayment<DOMAINROYALTIESPROPORTIONAL, FT>,
        collection: &mut nft_protocol::collection::Collection<DOMAINROYALTIESPROPORTIONAL>,
        ctx: &mut sui::tx_context::TxContext,
    ) {
        let b = nft_protocol::royalties::balance_mut(Witness {}, payment);

        let domain = nft_protocol::royalty::royalty_domain(collection);
        let royalty_owed =
            nft_protocol::royalty::calculate_proportional_royalty(domain, sui::balance::value(b));

        nft_protocol::royalty::collect_royalty(collection, b, royalty_owed);
        nft_protocol::royalties::transfer_remaining_to_beneficiary(Witness {}, payment, ctx);
    }

    fun mint(
        name: std::string::String,
        url: vector<u8>,
        mint_cap: &nft_protocol::mint_cap::MintCap<DOMAINROYALTIESPROPORTIONAL>,
        ctx: &mut sui::tx_context::TxContext,
    ): nft_protocol::nft::Nft<DOMAINROYALTIESPROPORTIONAL> {
        let nft = nft_protocol::nft::from_mint_cap(
            mint_cap,
            name,
            sui::url::new_unsafe_from_bytes(url),
            ctx,
        );
        let delegated_witness = nft_protocol::witness::from_witness<DOMAINROYALTIESPROPORTIONAL, Witness>(&Witness {});

        nft
    }
}
