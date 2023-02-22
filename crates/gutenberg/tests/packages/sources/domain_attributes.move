module gutenberg::domainattributes {
    use std::string::{Self, String};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use nft_protocol::nft::{Self, Nft};
    use nft_protocol::witness;
    use nft_protocol::mint_cap::MintCap;
    use nft_protocol::collection::{Self, Collection};
    use nft_protocol::display;
    use nft_protocol::creators;


    /// One time witness is only instantiated in the init method
    struct DOMAINATTRIBUTES has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}



    fun init(witness: DOMAINATTRIBUTES, ctx: &mut TxContext) {
        let (mint_cap, collection) = collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness(&Witness {});

        let creators = vec_set::empty();

        collection::add_domain(
            delegated_witness,
            &mut collection,
            creators::from_creators<DOMAINATTRIBUTES, Witness>(
                &Witness {}, creators,
            ),
        );

        
        transfer::transfer(mint_cap, tx_context::sender(ctx));
        transfer::share_object(collection);

    }



    public entry fun mint_to_address(
                attribute_keys: vector<String>,
        attribute_values: vector<String>,
        mint_cap: &MintCap<SUIMARINES>,
        receiver: address,
        ctx: &mut TxContext,
    ) {
        let nft = mint(
                        attribute_keys,
            attribute_values,
            mint_cap,
            ctx,
        );

        transfer::transfer(nft, receiver);
    }

    fun mint(
                attribute_keys: vector<String>,
        attribute_values: vector<String>,
        mint_cap: &MintCap<SUIMARINES>,
        ctx: &mut TxContext,
    ): Nft<DOMAINATTRIBUTES> {
        let nft = nft::from_mint_cap(mint_cap, name, url::new_unsafe_from_bytes(url), ctx);
        let delegated_witness = witness::from_witness(&Witness {});

                display::add_attributes_domain_from_vec(
            delegated_witness, &mut nft, attribute_keys, attribute_values,
        );

        nft
    }
}
