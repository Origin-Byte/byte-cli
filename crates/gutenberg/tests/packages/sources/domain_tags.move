module gutenberg::domaintags {
    use std::string::{Self, String};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use nft_protocol::nft::{Self, Nft};
    use nft_protocol::witness;
    use nft_protocol::mint_cap::MintCap;
    use nft_protocol::collection::{Self, Collection};
    use nft_protocol::tags;
    use nft_protocol::creators;


    /// One time witness is only instantiated in the init method
    struct DOMAINTAGS has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}



    fun init(witness: DOMAINTAGS, ctx: &mut TxContext) {
        let (mint_cap, collection) = collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness(&Witness {});

        let creators = vec_set::empty();

        collection::add_domain(
            delegated_witness,
            &mut collection,
            creators::from_creators<DOMAINTAGS, Witness>(
                &Witness {}, creators,
            ),
        );

        let tags = tags::empty(ctx);
        tags::add_tag(&mut tags, tags::art());
        tags::add_tag(&mut tags, tags::profile_picture());
        tags::add_collection_tag_domain(
            delegated_witness,
            &mut collection,
            tags,
        );

        transfer::transfer(mint_cap, tx_context::sender(ctx));
        transfer::share_object(collection);

    }



    public entry fun mint_to_address(
                mint_cap: &MintCap<SUIMARINES>,
        receiver: address,
        ctx: &mut TxContext,
    ) {
        let nft = mint(
                        mint_cap,
            ctx,
        );

        transfer::transfer(nft, receiver);
    }

    fun mint(
                mint_cap: &MintCap<SUIMARINES>,
        ctx: &mut TxContext,
    ): Nft<DOMAINTAGS> {
        let nft = nft::from_mint_cap(mint_cap, name, url::new_unsafe_from_bytes(url), ctx);
        let delegated_witness = witness::from_witness(&Witness {});

        
        nft
    }
}
