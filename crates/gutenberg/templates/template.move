module {package_name}::{module_name} {{
    /// One time witness is only instantiated in the init method
    struct {witness} has drop {{}}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {{}}

    {nft_struct}

{type_declarations}
{init_function}{entry_functions}
{tests}
}}
