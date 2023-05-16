//! Unit tests on input validation for the Schema struct

use std::collections::BTreeSet;

use gutenberg::{
    err::GutenError,
    models::{
        collection::CollectionData,
        launchpad::{listing::Listing, market::Market},
        settings::Composability,
        Address,
    },
};

// TODO:
// Input name: fail on non alpha-numeric
// Input creators: non addresses, hexadecimals with wrong lenghts, lack of 0x,
// empty vectors, etc. Listings > Markets > The token string field is currenly
// not being validated Schema should not write Move if no mintpolicy is
// selected..
// Input sanitation from the CLI >> avoid code injection
// Input creators: empty vectors
// Listings > Markets > The token string field is currenly not being validated

#[test]
fn input_name() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    collection.set_name(String::from("Suimarines"))?;
    assert_eq!(collection.name, String::from("suimarines"));

    collection.set_name(String::from("SUIMARINES"))?;
    assert_eq!(collection.name, String::from("suimarines"));

    collection.set_name(String::from("suimarines"))?;
    assert_eq!(collection.name, String::from("suimarines"));

    Ok(())
}

#[test]
fn input_description() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    collection
        .set_description(String::from("It supports non-alphanumeric &&&"));
    assert_eq!(
        collection.description.unwrap(),
        String::from("It supports non-alphanumeric &&&")
    );

    Ok(())
}

#[test]
fn input_symbol() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    collection.set_symbol(String::from("SUIM"))?;
    assert_eq!(*collection.symbol.as_ref().unwrap(), String::from("SUIM"));

    collection.set_symbol(String::from("suim"))?;
    assert_eq!(*collection.symbol.as_ref().unwrap(), String::from("SUIM"));

    Ok(())
}

#[test]
fn input_url() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    collection.set_url(String::from("http://originbyte.io"))?;
    assert_eq!(
        *collection.url.as_ref().unwrap(),
        String::from("http://originbyte.io")
    );

    collection.set_url(String::from("www.originbyte.io"))?;
    assert_eq!(
        *collection.url.as_ref().unwrap(),
        String::from("http://originbyte.io")
    );

    Ok(())
}

#[test]
fn input_creators() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    let creators = vec![String::from(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed",
    )];

    collection.set_creators(creators.clone())?;

    let creators_ = vec![Address::new(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed"
            .to_string(),
    )?];

    assert_eq!(collection.creators, creators_);

    let creators = vec![
        String::from("0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed"),
        String::from("0x582547ac2b368b17409a3f3672fe2eea9767b80830497fb2e31a15bc492f5516"),
        String::from("0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df"),
    ];

    collection.set_creators(creators.clone())?;

    let creators_ = vec![
        Address::new("0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed".to_string())?,
        Address::new("0x582547ac2b368b17409a3f3672fe2eea9767b80830497fb2e31a15bc492f5516".to_string())?,
        Address::new("0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df".to_string())?,
    ];
    assert_eq!(collection.creators, creators_);

    Ok(())
}

#[ignore]
#[test]
fn input_composability() -> Result<(), GutenError> {
    // let mut settings = Settings::default();

    let mut types = BTreeSet::new();

    types.insert(String::from("Avatar"));
    types.insert(String::from("Hat"));
    types.insert(String::from("Shoes"));

    let _composability =
        Composability::new_from_tradeable_traits(types, String::from("Avatar"));

    // settings.set_composability(composability.clone());
    //  assert_eq!(settings.composability.unwrap(), composability);

    Ok(())
}

#[test]
fn input_listing() -> Result<(), GutenError> {
    let admin = Address::new(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed"
            .to_string(),
    )?;
    let receiver = Address::new(
        "0x1225dd576b9fa621fb2aab078f82b88bf6c5a9260dbac34f7b1010917bd795ed"
            .to_string(),
    )?;

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

    Ok(())
}

#[test]
fn err_input_name_if_non_alphanumeric() -> Result<(), GutenError> {
    let mut collection = CollectionData::default();

    assert!(collection.set_name(String::from("Suimarine$")).is_err());
    assert!(collection.set_name(String::from("Suimarine#")).is_err());
    assert!(collection.set_name(String::from("Suimarine&")).is_err());
    assert!(collection.set_name(String::from("Suimarine/")).is_err());
    assert!(collection.set_name(String::from("Suimarine>")).is_err());
    assert!(collection.set_name(String::from("Suimarine<")).is_err());
    assert!(collection.set_name(String::from("Suimarine.")).is_err());
    assert!(collection.set_name(String::from("Suimarine_")).is_err());
    assert!(collection.set_name(String::from("Suimarine~")).is_err());
    assert!(collection.set_name(String::from("Suimarine^")).is_err());
    assert!(collection.set_name(String::from("Suimarine|")).is_err());

    Ok(())
}

#[test]
fn input_addresses() -> Result<(), GutenError> {
    // In converting a string to an address the function should be
    // able to handle hexadecimals with or without 0x prefix.
    let address =
        "0x1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df"
            .to_string();
    let address_2 =
        "1a4f2b04e99311b0ff8228cf12735402f6618d7be0f0b320364339baf03e49df"
            .to_string();

    assert_eq!(Address::new(address)?, Address::new(address_2)?);

    Ok(())
}

#[test]
fn err_input_incorrect_addresses() -> Result<(), GutenError> {
    assert!(Address::new(
        "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1432144152".to_string()
    )
    .is_err());
    assert!(Address::new(
        "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d1437d58fdc2eb880".to_string()
    )
    .is_err());
    assert!(Address::new("0xd8fb1b0".to_string()).is_err());
    assert!(Address::new("This is not an hexadecimal".to_string()).is_err());

    Ok(())
}
