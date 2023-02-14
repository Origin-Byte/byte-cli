use dialoguer::{theme::ColorfulTheme, MultiSelect};

use crate::prelude::CliError;

pub mod collection;
pub mod marketplace;
pub mod royalties;
pub mod nft;

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

fn map_indices(indices: Vec<usize>, arr: &[&str]) -> Vec<String> {
    let vec: Vec<String> = indices
        .iter()
        .map(|index| arr[*index].to_string())
        .collect();
    vec
}

pub fn multi_select<'a>(
    theme: &ColorfulTheme,
    prompt: &str,
    options: &[&'a str],
) -> anyhow::Result<Vec<String>> {
    let indexes = MultiSelect::with_theme(theme)
        .with_prompt(prompt)
        .items(options)
        .interact()
        .unwrap();

    let borrowed = indexes
        .iter()
        .map(|i| options[*i].to_string())
        .collect::<Vec<_>>();

    Ok(borrowed)
}

fn get_options<'a>(
    theme: &ColorfulTheme,
    prompt: &str,
    options: &[&'a str],
) -> anyhow::Result<Vec<String>> {
    let mut chosen_opts = multi_select(theme, prompt, options)?;

    while chosen_opts.len() == 0 {
        println!("You have to select at least one option.");
        chosen_opts = multi_select(theme, prompt, options)?;
    }

    Ok(chosen_opts)
}
