module gutenberg::domaindisplay {
    use std::string::{Self, String};
    use sui::transfer;
    use sui::tx_context::{Self, TxContext};
    use nft_protocol::nft::{Self, Nft};
    use nft_protocol::witness;
    use nft_protocol::mint_cap::{MintCap};
    use nft_protocol::collection::{Self, Collection};
    use nft_protocol::display;
    use nft_protocol::creators;


    /// One time witness is only instantiated in the init method
    struct DOMAINDISPLAY has drop {}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {}



    fun init(witness: DOMAINDISPLAY, ctx: &mut TxContext) {
        let (mint_cap, collection) = collection::create(&witness, ctx);
        let delegated_witness = nft_protocol::witness::from_witness(&Witness {});

        let creators = vec_set::empty();

        collection::add_domain(
            delegated_witness,
            &mut collection,
            creators::from_creators<DOMAINDISPLAY, Witness>(
                &Witness {}, creators,
            ),
        );

            display::add_collection_display_domain(
                delegated_witness,
                &mut collection,
                string::utf8(b"DomainDisplay"),
                string::utf8(b"Test contract generated by Gutenberg"),
            );

        
        transfer::transfer(mint_cap, tx_context::sender(ctx));
        transfer::share_object(collection);

    }



    public entry fun mint_to_address(
        name: String,
        description: String,
        mint_cap: &MintCap<SUIMARINES>,
        receiver: address,        ctx: &mut TxContext,

    ) {
        let nft = mint(
            name,
            description,
            mint_cap,
            ctx,
        );

        transfer::transfer(nft, receiver);
    }

    fun mint(
        name: String,
        description: String,
        mint_cap: &MintCap<SUIMARINES>,
        ctx: &mut TxContext,
        ): Nft<DOMAINDISPLAY> {
        let nft = nft::from_mint_cap(mint_cap, name, url::new_unsafe_from_bytes(url), ctx);
        let delegated_witness = witness::from_witness(&Witness {});

        display::add_display_domain(
            delegated_witness, &mut nft, name, description,
        );

        nft
    }
}
