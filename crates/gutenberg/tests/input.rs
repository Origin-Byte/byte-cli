//! Unit tests on input validation for the Schema struct

use std::collections::BTreeSet;

use gutenberg::models::{
    collection::CollectionData,
    launchpad::{listing::Listing, market::Market},
    settings::{Composability, Settings},
    Address,
};

use anyhow::Result;

// TODO:
// Input sanitation from the CLI >> avoid code injection
// Input creators: empty vectors
// Listings > Markets > The token string field is currenly not being validated

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

    let creators_ = vec![Address::new(
        "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string(),
    )?];

    assert_eq!(collection.creators, creators_);

    let creators = vec![
        String::from("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string()),
        String::from("0xa7e29665a1c2600a439de3316b76e0c8a7531916".to_string()),
        String::from("0x755a088ca3847eebc66103e1dea89845e306fe46".to_string()),
    ];

    collection.set_creators(creators.clone())?;

    let creators_ = vec![
        Address::new("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string())?,
        Address::new("0xa7e29665a1c2600a439de3316b76e0c8a7531916".to_string())?,
        Address::new("0x755a088ca3847eebc66103e1dea89845e306fe46".to_string())?,
    ];
    assert_eq!(collection.creators, creators_);

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
fn input_listing() -> Result<()> {
    let admin =
        Address::new("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string())?;
    let receiver =
        Address::new("0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string())?;

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
fn err_input_name_if_non_alphanumeric() -> Result<()> {
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
fn input_addresses() -> Result<()> {
    // In converting a string to an address the function should be
    // able to handle hexadecimals with or without 0x prefix.
    let address = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string();
    let address_2 = "d8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143".to_string();

    assert_eq!(Address::new(address)?, Address::new(address_2)?);

    Ok(())
}

#[test]
fn err_input_incorrect_addresses() -> Result<()> {
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
