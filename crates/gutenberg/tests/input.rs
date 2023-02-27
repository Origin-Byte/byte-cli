//! Unit tests on input validation for the Schema struct

use std::collections::BTreeSet;

use gutenberg::models::{
    collection::CollectionData,
    launchpad::{listing::Listing, market::Market},
    settings::{Composability, Settings},
};

use anyhow::Result;

// TODO:
// Input name: fail on non alpha-numeric
// Input creators: non addresses, hexadecimals with wrong lenghts, lack of 0x, empty vectors, etc.
// Listings > Markets > The token string field is currenly not being validated
// Schema should not write Move if no mintpolicy is selected..

// Base tests
// name
// description
// symbol
// url
// creators
// composability
// listing

#[test]
fn input_name() -> Result<()> {
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
fn input_description() -> Result<()> {
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
fn input_symbol() -> Result<()> {
    let mut collection = CollectionData::default();

    collection.set_symbol(String::from("SUIM"))?;
    assert_eq!(*collection.symbol.as_ref().unwrap(), String::from("SUIM"));

    collection.set_symbol(String::from("suim"))?;
    assert_eq!(*collection.symbol.as_ref().unwrap(), String::from("SUIM"));

    Ok(())
}

#[test]
fn input_url() -> Result<()> {
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
fn input_creators() -> Result<()> {
    let mut collection = CollectionData::default();

    let creators =
        vec![String::from("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143")];

    collection.set_creators(creators.clone())?;
    assert_eq!(collection.creators, creators);

    let creators = vec![
        String::from("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143"),
        String::from("0xa7e29665a1c2600a439de3316b76e0c8a7531916"),
        String::from("0x755a088ca3847eebc66103e1dea89845e306fe46"),
    ];

    collection.set_creators(creators.clone())?;
    assert_eq!(collection.creators, creators);

    Ok(())
}

#[test]
fn input_composability() -> Result<()> {
    let mut settings = Settings::default();

    let mut types = BTreeSet::new();

    types.insert(String::from("Avatar"));
    types.insert(String::from("Hat"));
    types.insert(String::from("Shoes"));

    let composability =
        Composability::new_from_tradeable_traits(types, String::from("Avatar"));

    settings.set_composability(composability.clone());
    assert_eq!(settings.composability.unwrap(), composability);

    Ok(())
}

#[test]
fn input_listing() {
    let admin = String::from("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143");
    let receiver = String::from("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143");

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
