//! Unit tests on input validation for the Schema struct

// TODO:
// Input name: fail on non alpha-numeric
// Input creators: non addresses, hexadecimals with wrong lenghts, lack of 0x,
// empty vectors, etc. Listings > Markets > The token string field is currenly
// not being validated Schema should not write Move if no mintpolicy is
// selected..
// Input sanitation from the CLI >> avoid code injection
// Input creators: empty vectors
// Listings > Markets > The token string field is currenly not being validated

use gutenberg_types::models::{
    collection::Address,
    launchpad::{listing::Listing, market::Market},
};

#[test]
fn input_listing() {
    let admin = Address::new(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed",
    )
    .unwrap();
    let receiver = Address::new(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed",
    )
    .unwrap();

    let mut markets = Vec::new();

    let price = 100;
    let is_whitelisted = false;

    let market_1 = Market::FixedPrice {
        token: "sui::sui::SUI".to_string(),
        price,
        is_whitelisted,
    };

    markets.push(market_1.clone());

    let reserve_price = 500;
    let is_whitelisted = true;
    let market_2 = Market::DutchAuction {
        token: "sui::sui::SUI".to_string(),
        reserve_price,
        is_whitelisted,
    };

    markets.push(market_2.clone());

    let listing = Listing::new(admin.clone(), receiver.clone(), markets);

    assert_eq!(listing.admin, admin);
    assert_eq!(listing.receiver, receiver);
    assert_eq!(listing.markets[0], market_1);
    assert_eq!(listing.markets[1], market_2);
}
