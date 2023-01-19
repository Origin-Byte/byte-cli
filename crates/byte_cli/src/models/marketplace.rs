use crate::prelude::get_dialoguer_theme;

use super::{address_validator, number_validator, sender, FromPrompt};
use dialoguer::{Confirm, Input, Select};
use gutenberg::models::marketplace::{Listing, Listings, Market, Marketplace};

const MARKET_OPTIONS: [&str; 2] = ["Fixed price", "Dutch auction"];

impl FromPrompt for Marketplace {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let admin = Input::with_theme(&theme)
            .with_prompt("What is the address of the listing administrator?")
            .default(sender().to_string())
            .validate_with(address_validator)
            .interact()
            .unwrap();

        let receiver = Input::with_theme(&theme)
            .with_prompt("What is the address that receives the sale proceeds?")
            .default(sender().to_string())
            .validate_with(address_validator)
            .interact()
            .unwrap();

        Ok(Marketplace { admin, receiver })
    }
}

impl FromPrompt for Market {
    fn from_prompt() -> Result<Self, anyhow::Error>
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

        Ok(market)
    }
}

impl FromPrompt for Listing {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        Ok(Listing::new(Market::from_prompt()?))
    }
}

impl FromPrompt for Listings {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();
        let mut listings = Listings::default();

        let number = Input::with_theme(&theme)
            .with_prompt(
                // TODO: The meaning of this questions may be ambiguous
                // from the perspective of the creator
                "How many different markets do you plan on having?",
            )
            .validate_with(number_validator)
            .interact()?
            .parse::<u64>()?;

        for _ in 0..number {
            listings.0.push(Listing::from_prompt()?);
        }

        Ok(listings)
    }
}
