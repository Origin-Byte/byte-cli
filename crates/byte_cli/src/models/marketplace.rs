use crate::prelude::get_dialoguer_theme;

use super::{address_validator, sender, FromPrompt};
use dialoguer::Input;
use gutenberg::models::marketplace::Marketplace;

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
