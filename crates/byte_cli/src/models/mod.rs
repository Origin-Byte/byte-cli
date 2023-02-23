use dialoguer::{theme::ColorfulTheme, MultiSelect};
use gutenberg::Schema;

use crate::{
    consts::{MAX_SYMBOL_LENGTH, TX_SENDER_ADDRESS},
    prelude::CliError,
};

pub mod collection;
pub mod marketplace;
pub mod nft;
pub mod royalties;
pub mod settings;
pub mod sui_output;

/// Trait for constructing Gutenberg objects from prompt
pub trait FromPrompt {
    fn from_prompt(schema: &Schema) -> Result<Option<Self>, anyhow::Error>
    where
        Self: Sized;
}

pub fn bps_validator(input: &String) -> Result<(), String> {
    if input.parse::<u64>().is_err() {
        Err(format!("Couldn't parse '{input}' to a number."))
    } else {
        if input.parse::<u64>().unwrap() > 10_000 {
            Err(format!(
                "The Basis Points number {input} provided is above 100%."
            ))
        } else {
            Ok(())
        }
    }
}

pub fn url_validator(input: &String) -> Result<(), String> {
    if input.parse::<String>().is_err() {
        Err(format!("Couldn't parse '{input}' to a string."))
    } else {
        let url_ = input.parse::<String>().unwrap().to_string();
        let mut url: String;

        if url_.starts_with("www.") {
            url = String::from("http://");
            url.push_str(url_.split_at(4).0);
        } else {
            url = url_;
        }

        if url::Url::parse(&url).is_err() {
            let err = url::Url::parse(&url);
            Err(format!(
                "The following error has occured: {:?}
        The Collection URL input `{}` is not valid.",
                err, url
            ))
        } else {
            Ok(())
        }
    }
}

pub fn name_validator(input: &String) -> Result<(), String> {
    if input.parse::<String>().is_err() {
        Err(format!("Couldn't parse '{input}' to a string."))
    } else {
        let name = input.parse::<String>().unwrap();

        if !name.chars().all(|c| c.is_ascii_alphanumeric()) {
            Err(format!(
                "The collection name provided `{}` should only have alphanumeric characters.",
                name
            ))
        } else {
            Ok(())
        }
    }
}

pub fn symbol_validator(input: &String) -> Result<(), String> {
    if input.parse::<String>().is_err() {
        Err(format!("Couldn't parse '{input}' to a string."))
    } else {
        let symbol = input.parse::<String>().unwrap();
        if symbol.len() > MAX_SYMBOL_LENGTH as usize {
            Err(format!(
                "The symbol length {input} provided should not be bigger than {}.",
                MAX_SYMBOL_LENGTH
            ))
        } else {
            Ok(())
        }
    }
}

pub fn positive_integer_validator(input: &String) -> Result<(), String> {
    if input.parse::<u64>().is_err() {
        Err(format!("Couldn't parse '{input}' to a number."))
    } else {
        let numb = input.parse::<u64>().unwrap();
        if numb <= 0 {
            Err(format!(
                "The number {input} provided has to be bigger than 0."
            ))
        } else if numb > 20 {
            Err(format!(
                "The number {input} provided is above the limit of 20."
            ))
        } else {
            Ok(())
        }
    }
}

pub fn number_validator(input: &String) -> Result<(), String> {
    if input.parse::<u64>().is_err() {
        Err(format!("Couldn't parse '{input}' to a number."))
    } else {
        Ok(())
    }
}

pub fn address_validator(input: &String) -> Result<(), CliError> {
    if input == TX_SENDER_ADDRESS {
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
