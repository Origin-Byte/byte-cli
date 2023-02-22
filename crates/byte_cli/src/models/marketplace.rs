use crate::{
    consts::{MARKET_OPTIONS, TX_SENDER_ADDRESS},
    prelude::get_dialoguer_theme,
};

use super::{address_validator, number_validator, FromPrompt};
use dialoguer::{Confirm, Input, Select};
use gutenberg::{
    models::marketplace::{Listing, Listings, Market},
    Schema,
};

impl FromPrompt for Market {
    fn from_prompt(_scheme: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let market_index = Select::with_theme(&theme)
            .with_prompt(
                "What is the market primitive to use for the next sale?",
            )
            .items(&MARKET_OPTIONS)
            .interact()?;

        let is_whitelisted = Confirm::with_theme(&theme)
            .with_prompt("Is it a whitelisted sale?")
            .interact()?;

        let market = match MARKET_OPTIONS[market_index] {
            "Fixed price" => {
                let price = Input::with_theme(&theme)
                    .with_prompt("What is the price of the sale?")
                    .validate_with(number_validator)
                    .interact()?
                    .parse::<u64>()?;

                Market::FixedPrice {
                    token: "sui::sui::SUI".to_string(),
                    price,
                    is_whitelisted,
                }
            }
            "Dutch auction" => {
                let reserve_price = Input::with_theme(&theme)
                    .with_prompt("What is the reserve price of the auction?")
                    .validate_with(number_validator)
                    .interact()?
                    .parse::<u64>()?;

                Market::DutchAuction {
                    token: "sui::sui::SUI".to_string(),
                    reserve_price,
                    is_whitelisted,
                }
            }
            _ => unreachable!(),
        };

        Ok(Some(market))
    }
}

impl FromPrompt for Listing {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let number = Input::with_theme(&theme)
            .with_prompt(
                // TODO: Refer to what listing we are talking about..
                "How many sale venues do you want to create? (Note: One listing can have multiple venues with different configurations)",
            )
            .validate_with(number_validator)
            .interact()?
            .parse::<u64>()?;

        let mut markets = vec![];

        for _ in 0..number {
            markets.push(Market::from_prompt(schema)?.unwrap());
        }

        Ok(Some(Listing::new(markets)))
    }
}

impl FromPrompt for Listings {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();
        let mut listings = Listings::default();

        let admin = Input::with_theme(&theme)
            .with_prompt("What is the address of the listing administrator?")
            .default(String::from(TX_SENDER_ADDRESS))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        let receiver = Input::with_theme(&theme)
            .with_prompt("What is the address that receives the sale proceeds?")
            .default(String::from(TX_SENDER_ADDRESS))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        let number = Input::with_theme(&theme)
            .with_prompt(
                "How many listings do you plan on having? Click [here](https://docs.originbyte.io/origin-byte/about-our-programs/launchpad#listing) to learn more about listings.",
            )
            .default("1".to_string())
            .validate_with(number_validator)
            .interact()?
            .parse::<u64>()?;

        for _ in 0..number {
            listings.0.push(Listing::from_prompt(schema)?.unwrap());
        }

        for listing in listings.0.iter_mut() {
            listing.admin = admin.clone();
            listing.receiver = receiver.clone();
        }

        Ok(Some(listings))
    }
}
