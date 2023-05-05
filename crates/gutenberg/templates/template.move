module {module_alias}::{module_name} {{

    use std::ascii;
    use std::option;
    use std::string::{Self, String};

    use sui::url::{Self, Url};
    use sui::sui::SUI;
    use sui::display;
    use sui::transfer;
    use sui::object::{Self, UID};
    use sui::tx_context::{Self, TxContext};

    use nft_protocol::mint_cap;
    use nft_protocol::mint_event;
    use nft_protocol::creators;
    use nft_protocol::attributes::{Self, Attributes};
    use nft_protocol::collection;
    use nft_protocol::display_info;
    use nft_protocol::mint_cap::MintCap;
    use nft_protocol::royalty;
    use nft_protocol::royalty_strategy_bps;
    use nft_protocol::tags;
    use nft_protocol::transfer_allowlist;

    use ob_utils::utils;
    use ob_utils::display as ob_display;

    use ob_permissions::witness;
    use ob_request::transfer_request;
    use ob_launchpad::warehouse::{Self, Warehouse};

    use ob_allowlist::allowlist;

    use liquidity_layer_v1::orderbook;
    use liquidity_layer_v1::bidding;

    /// One time witness is only instantiated in the init method
    struct {witness} has drop {{}}

    /// Can be used for authorization of other actions post-creation. It is
    /// vital that this struct is not freely given to any contract, because it
    /// serves as an auth token.
    struct Witness has drop {{}}
{type_declarations}
{init_function}{entry_functions}
}}
