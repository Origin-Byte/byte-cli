module gutenberg::domaincreators {
    use std::string::{Self, String};
    use sui::vec_set;
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use nft_protocol::nft::{Self, Nft};
    use nft_protocol::witness;
    use nft_protocol::mint_cap::{MintCap};
    use nft_protocol::collection::{Self, Collection};
    use nft_protocol::creators;


    /// One time witness is only instantiated in the init method
    struct DOMAINCREATORS has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}



    fun init(witness: DOMAINCREATORS, ctx: &mut TxContext) {
        let (mint_cap, collection) = collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness(&Witness {});

        
        collection::add_domain(
            delegated_witness,
            &mut collection,
            
    creators::from_address<DomainCreators, Witness>(
        &Witness {}, 0x64be9c21161c2305543e2bba67e056ebba8729e4, ctx,
    ),
        );

        
        transfer::transfer(mint_cap, tx_context::sender(ctx));
        transfer::share_object(collection);

    }



    public entry fun mint_to_address(
                mint_cap: &MintCap<SUIMARINES>,
        receiver: address,        ctx: &mut TxContext,

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
        ): Nft<DOMAINCREATORS> {
        let nft = nft::from_mint_cap(mint_cap, name, url::new_unsafe_from_bytes(url), ctx);
        let delegated_witness = witness::from_witness(&Witness {});

        
        nft
    }
}
