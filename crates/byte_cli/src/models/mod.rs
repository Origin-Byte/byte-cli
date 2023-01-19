use crate::prelude::CliError;

pub mod marketplace;
pub mod royalties;

/// Trait for constructing Gutenberg objects from prompt
pub trait FromPrompt {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub fn sender() -> &'static str {
    "tx_context::sender(ctx)"
}

pub fn number_validator(input: &String) -> Result<(), String> {
    if input.parse::<u64>().is_err() {
        Err(format!("Couldn't parse '{input}' to a number."))
    } else {
        Ok(())
    }
}

pub fn address_validator(input: &String) -> Result<(), CliError> {
    if input == sender() {
        return Ok(());
    }

    let hexa_str = input.strip_prefix("0x").unwrap_or(input);
    let hexa = hex::decode(hexa_str)?;
    if hexa.len() != 20 {
        Err(CliError::InvalidAddressLength)
    } else {
        Ok(())
    }
}
