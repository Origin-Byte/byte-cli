module {module_name}::{module_name} {{
    {imports}

    /// One time witness is only instantiated in the init method
    struct {witness} has drop {{}}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {{}}

    {type_declarations}

    {init_function}

    /// Calculates and transfers royalties to the `RoyaltyDomain`
    public entry fun collect_royalty<FT>(
        payment: &mut TradePayment<{witness}, FT>,
        collection: &mut Collection<{witness}>,
        ctx: &mut TxContext,
    ) {{
        let b = royalties::balance_mut(Witness {{}}, payment);

        let domain = royalty::royalty_domain(collection);
        let royalty_owed =
            royalty::calculate_proportional_royalty(domain, balance::value(b));

        royalty::collect_royalty(collection, b, royalty_owed);
        royalties::transfer_remaining_to_beneficiary(Witness {{}}, payment, ctx);
    }}

    {mint_functions}
}}
