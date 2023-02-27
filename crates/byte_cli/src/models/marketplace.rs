use crate::{
    consts::{
        DEFAULT_SENDER_MSG, MARKET_OPTIONS, MARKET_OPTIONS_, TX_SENDER_ADDRESS,
    },
    models::bps_validator,
    prelude::get_dialoguer_theme,
};

use super::{address_validator, number_validator, FromPrompt};
use console::style;
use dialoguer::{Confirm, Input, Select};
use gutenberg::{
    models::launchpad::{
        listing::{Listing, Listings},
        market::Market,
        marketplace::Marketplace,
    },
    Schema,
};

impl FromPrompt for Marketplace {
    fn from_prompt(_schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut admin = Input::with_theme(&theme)
            .with_prompt("Please provide an administrator address:")
            .default(String::from(DEFAULT_SENDER_MSG))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        if admin == DEFAULT_SENDER_MSG.to_string() {
            admin = TX_SENDER_ADDRESS.to_string();
        }

        let mut receiver = Input::with_theme(&theme)
            .with_prompt("Please provide a proceeds receiver address:")
            .default(String::from(TX_SENDER_ADDRESS))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        if receiver == DEFAULT_SENDER_MSG.to_string() {
            receiver = TX_SENDER_ADDRESS.to_string();
        }

        let default_fee = Input::with_theme(&theme)
            .with_prompt(
                "Please provide a default fee policy, in basis points:",
            )
            .validate_with(bps_validator)
            .interact()
            .unwrap()
            .parse::<u64>()
            .unwrap();

        let marketplace = Marketplace::new(admin, receiver, default_fee);

        println!(
            "{}",
            style(
                "Looks like we're all set! Your on-chain marketplace is configured."
            )
            .blue()
            .bold()
            .dim()
        );

        Ok(Some(marketplace))
    }
}

impl FromPrompt for Market {
    fn from_prompt(_scheme: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let market_index = Select::with_theme(&theme)
            .with_prompt("What is the market primitive to use?")
            .items(&MARKET_OPTIONS)
            .interact()?;

        let is_whitelisted = Confirm::with_theme(&theme)
            .with_prompt("Is it a whitelisted sale?")
            .interact()?;

        let market = match MARKET_OPTIONS_[market_index] {
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

        let mut admin = Input::with_theme(&theme)
            .with_prompt("What is the address of the listing administrator?")
            .default(String::from(DEFAULT_SENDER_MSG))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        if admin == DEFAULT_SENDER_MSG.to_string() {
            admin = TX_SENDER_ADDRESS.to_string();
        }

        let mut receiver = Input::with_theme(&theme)
            .with_prompt("What is the address that receives the sale proceeds?")
            .default(String::from(DEFAULT_SENDER_MSG))
            .validate_with(address_validator)
            .interact()
            .unwrap();

        if receiver == DEFAULT_SENDER_MSG.to_string() {
            receiver = TX_SENDER_ADDRESS.to_string();
        }

        let number = Input::with_theme(&theme)
            .with_prompt(
                "How many sale venues do you want to create? (Note: One listing can have multiple venues with different configurations)",
            )
            .validate_with(number_validator)
            .interact()?
            .parse::<u64>()?;

        let mut markets = vec![];

        for i in 0..number {
            println!(
                "{}",
                style(format!("Let's setup the sale venue number {}.", i + 1))
                    .blue()
                    .bold()
                    .dim()
            );

            markets.push(Market::from_prompt(schema)?.unwrap());
        }

        Ok(Some(Listing::new(admin, receiver, markets)))
    }
}
