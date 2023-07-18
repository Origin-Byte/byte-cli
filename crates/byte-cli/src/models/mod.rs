pub mod collection;
pub mod effects;
pub mod nft;
pub mod royalties;

use crate::{
    consts::{MAX_SYMBOL_LENGTH, TX_SENDER_ADDRESS},
    err::CliError,
};
use dialoguer::{theme::ColorfulTheme, MultiSelect};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Accounts {
    pub main: Option<String>,
    pub accounts: Vec<Account>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Account {
    pub email: String,
    pub password: String,
}

impl Accounts {
    pub fn new(main: String, accounts: Vec<Account>) -> Self {
        Self {
            main: Some(main),
            accounts,
        }
    }

    pub fn get_main_account(&self) -> &Account {
        self.accounts
            .iter()
            .find(|account| {
                account.email.as_str()
                    == self.main.as_ref().expect("Failed to get main account")
            })
            .expect("Failed to get find main account in accounts list.")
    }

    pub fn register_if_not_yet(&mut self, email: &str, password: &str) {
        if self.check_if_registered(email) == false {
            self.accounts.push(Account {
                email: email.to_string(),
                password: password.to_string(),
            });
        }
    }

    pub fn check_if_registered(&self, email: &str) -> bool {
        self.accounts.iter().any(|account| &account.email == email)
    }

    pub fn set_main(&mut self, email: String) {
        self.main = Some(email);
    }

    pub fn set_main_if_none(&mut self, email: String) {
        if self.main.is_none() {
            self.main = Some(email);
        }
    }
}

impl Account {
    pub fn new(email: String, password: String) -> Self {
        Self { email, password }
    }
}

/// Trait for constructing Gutenberg objects from prompt
pub trait FromPrompt {
    type Param<'a>;

    fn from_prompt(param: Self::Param<'_>) -> Result<Self, anyhow::Error>
    where
        Self: Sized;
}

pub fn bps_validator(input: &String) -> Result<(), String> {
    if input.parse::<u64>().is_err() {
        Err(format!("Couldn't parse '{input}' to a number."))
    } else if input.parse::<u64>().unwrap() > 10_000 {
        Err(format!(
            "The Basis Points number {input} provided is above 100%."
        ))
    } else {
        Ok(())
    }
}

pub fn url_validator(input: &String) -> Result<(), String> {
    if input.parse::<String>().is_err() {
        Err(format!("Couldn't parse '{input}' to a string."))
    } else {
        let url_ = input.parse::<String>().unwrap();
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
        if numb == 0 {
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
    if hexa.len() != 32 {
        Err(CliError::InvalidAddressLength)
    } else {
        Ok(())
    }
}

fn map_indices<'a>(indices: Vec<usize>, arr: &[&'a str]) -> Vec<&'a str> {
    indices.iter().map(|index| arr[*index]).collect()
}

pub fn multi_select<'a>(
    theme: &ColorfulTheme,
    prompt: &str,
    option_fields: &[&'a str],
    option_values: &[&'a str],
) -> anyhow::Result<Vec<String>> {
    let indexes = MultiSelect::with_theme(theme)
        .with_prompt(prompt)
        .items(option_fields)
        .interact()
        .unwrap();

    let borrowed = indexes
        .iter()
        .map(|i| option_values[*i].to_string())
        .collect::<Vec<_>>();

    Ok(borrowed)
}

// This will be most likely added back
#[allow(dead_code)]
fn get_options<'a>(
    theme: &ColorfulTheme,
    prompt: &str,
    options_fields: &[&'a str],
    options_values: &[&'a str],
) -> anyhow::Result<Vec<String>> {
    let mut chosen_opts =
        multi_select(theme, prompt, options_fields, options_values)?;

    while chosen_opts.is_empty() {
        println!("You have to select at least one option.");
        chosen_opts =
            multi_select(theme, prompt, options_fields, options_values)?;
    }

    Ok(chosen_opts)
}
