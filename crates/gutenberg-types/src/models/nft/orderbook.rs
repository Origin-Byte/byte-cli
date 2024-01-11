use serde::{Deserialize, Serialize};

use super::Fields;

/// An enum representing the types of orderbooks available.
/// This is serialized and deserialized using `serde`.
#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Orderbook {
    Unprotected,
    Protected,
}


impl Orderbook {
    /// Generates a Move initialization code string for the orderbook based on its type.
    /// This function is used to create the necessary initialization code for an orderbook,
    /// which varies depending on whether it is `Unprotected` or `Protected`.
    ///
    /// # Arguments
    /// * `type_name` - The name of the type for which the orderbook is being initialized.
    pub fn write_move_init(&self, type_name: &str) -> String {
        match self {
            Orderbook::Unprotected => format!(
                "

        liquidity_layer_v1::orderbook::create_unprotected<{type_name}, sui::sui::SUI>(
            delegated_witness, &transfer_policy, ctx,
        );"
            ),
            Orderbook::Protected => format!(
                "

        // Protected orderbook such that trading is not initially possible
        let orderbook = liquidity_layer_v1::orderbook::new_with_protected_actions<{type_name}, sui::sui::SUI>(
            delegated_witness, &transfer_policy, liquidity_layer_v1::orderbook::custom_protection(true, true, true), ctx,
        );
        liquidity_layer_v1::orderbook::share(orderbook);"
            ),
        }
    }

    /// Creates Move test code as a string for the orderbook.
    /// This method generates test cases for the orderbook based on its configuration,
    /// and the provided `Fields`, `type_name`, `witness_name`, and conditions like
    /// `requires_collection` and `requires_royalties`.
    ///
    /// # Arguments
    /// * `fields` - Field definitions for the orderbook.
    /// * `type_name` - The name of the type associated with the orderbook.
    /// * `witness_name` - The name of the witness for the test.
    /// * `requires_collection` - Boolean indicating if collection is required.
    /// * `requires_royalties` - Boolean indicating if royalties are required.
    pub fn write_move_tests(
        &self,
        fields: &Fields,
        type_name: &str,
        witness_name: &str,
        requires_collection: bool,
        requires_royalties: bool,
    ) -> String {
        let collection_take_str = requires_collection.then(|| format!("

        let collection = sui::test_scenario::take_shared<nft_protocol::collection::Collection<{type_name}>>(
            &scenario,
        );")).unwrap_or_default();

        let collection_param_str = requires_collection
            .then_some(
                "
            &mut collection,",
            )
            .unwrap_or_default();

        let collection_return_str = requires_collection
            .then_some(
                "
        sui::test_scenario::return_shared(collection);",
            )
            .unwrap_or_default();

        let royalties_str = requires_royalties
            .then(|| format!("

        let royalty_strategy = sui::test_scenario::take_shared<nft_protocol::royalty_strategy_bps::BpsRoyaltyStrategy<{type_name}>>(&mut scenario);
        nft_protocol::royalty_strategy_bps::confirm_transfer<{type_name}, sui::sui::SUI>(&mut royalty_strategy, &mut request);
        sui::test_scenario::return_shared(royalty_strategy);"))
            .unwrap_or_default();

        let fields_str: String = fields
            .test_params()
            .map(|param| {
                format!(
                    "
            {param},"
                )
            })
            .collect();

        format!("

    #[test]
    fun test_trade() {{
        let scenario = sui::test_scenario::begin(CREATOR);

        init({witness_name} {{}}, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        // Setup allowlist
        let (allowlist, allowlist_cap) = ob_allowlist::allowlist::new(sui::test_scenario::ctx(&mut scenario));
        ob_allowlist::allowlist::insert_authority<liquidity_layer_v1::orderbook::Witness>(&allowlist_cap, &mut allowlist);

        let publisher = sui::test_scenario::take_from_address<sui::package::Publisher>(
            &scenario, CREATOR,
        );

        // Need to insert all tradeable types into collection
        ob_allowlist::allowlist::insert_collection<{type_name}>(&mut allowlist, &publisher);
        sui::transfer::public_share_object(allowlist);
        sui::transfer::public_transfer(allowlist_cap, CREATOR);

        // Setup orderbook
        let orderbook = sui::test_scenario::take_shared<liquidity_layer_v1::orderbook::Orderbook<{type_name}, sui::sui::SUI>>(&scenario);
        liquidity_layer_v1::orderbook::enable_orderbook(&publisher, &mut orderbook);

        sui::test_scenario::return_to_address(CREATOR, publisher);

        // Setup test NFT
        let mint_cap = sui::test_scenario::take_from_address<nft_protocol::mint_cap::MintCap<{type_name}>>(
            &mut scenario, CREATOR,
        );{collection_take_str}

        let nft = mint({fields_str}
            &mut mint_cap,{collection_param_str}
            sui::test_scenario::ctx(&mut scenario),
        );
        let nft_id = sui::object::id(&nft);

        sui::test_scenario::return_to_address(CREATOR, mint_cap);{collection_return_str}

        // Deposit NFT into Kiosk
        let (kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));
        ob_kiosk::ob_kiosk::deposit(&mut kiosk, nft, sui::test_scenario::ctx(&mut scenario));
        sui::transfer::public_share_object(kiosk);

        sui::test_scenario::next_tx(&mut scenario, CREATOR);

        // Test trade
        let seller_kiosk = sui::test_scenario::take_shared<sui::kiosk::Kiosk>(&scenario);
        let (buyer_kiosk, _) = ob_kiosk::ob_kiosk::new(sui::test_scenario::ctx(&mut scenario));

        liquidity_layer_v1::orderbook::create_ask(
            &mut orderbook,
            &mut seller_kiosk,
            100_000_000,
            nft_id,
            sui::test_scenario::ctx(&mut scenario),
        );

        let coin = sui::coin::mint_for_testing<sui::sui::SUI>(100_000_000, sui::test_scenario::ctx(&mut scenario));

        let trade_opt = liquidity_layer_v1::orderbook::create_bid(
            &mut orderbook,
            &mut buyer_kiosk,
            100_000_000,
            &mut coin,
            sui::test_scenario::ctx(&mut scenario),
        );

        sui::coin::burn_for_testing(coin);
        let trade = std::option::destroy_some(trade_opt);

        let request = liquidity_layer_v1::orderbook::finish_trade(
            &mut orderbook,
            liquidity_layer_v1::orderbook::trade_id(&trade),
            &mut seller_kiosk,
            &mut buyer_kiosk,
            sui::test_scenario::ctx(&mut scenario),
        );

        let allowlist = sui::test_scenario::take_shared<ob_allowlist::allowlist::Allowlist>(&scenario);
        nft_protocol::transfer_allowlist::confirm_transfer(&allowlist, &mut request);
        sui::test_scenario::return_shared(allowlist);{royalties_str}

        let transfer_policy = sui::test_scenario::take_shared<sui::transfer_policy::TransferPolicy<{type_name}>>(&scenario);
        ob_request::transfer_request::confirm<{type_name}, sui::sui::SUI>(request, &transfer_policy, sui::test_scenario::ctx(&mut scenario));
        sui::test_scenario::return_shared(transfer_policy);

        ob_kiosk::ob_kiosk::assert_nft_type<{type_name}>(&buyer_kiosk, nft_id);

        sui::transfer::public_share_object(buyer_kiosk);
        sui::test_scenario::return_shared(seller_kiosk);
        sui::test_scenario::return_shared(orderbook);
        sui::test_scenario::end(scenario);
    }}")
    }
}
