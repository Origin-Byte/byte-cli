use std::path::Path;

use crate::{
    cli::get_dialoguer_theme,
    io::LocalWrite,
    models::{Account, Accounts},
};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};
use gutenberg_types::models::{collection::CollectionData, nft::NftData};
use reqwest::{Body, Client, Response};
use rust_sdk::models::project::Project;
use serde_json::json;

pub async fn login(accounts: &mut Accounts) -> Result<Response> {
    let theme = get_dialoguer_theme();
    println!("{}", style("Welcome to SuiPlay!").blue().bold().dim());

    let email: String = Input::with_theme(&theme)
        .with_prompt("E-mail:")
        .interact()
        .unwrap();

    let password = Password::with_theme(&theme)
        .with_prompt("Password")
        .interact()
        .unwrap();

    let client = Client::new();

    let req_body = json!({
        "email": email,
        "password": password,
    });

    let res = client
        .post("https://suiplay-api-1o7v724t.ew.gateway.dev/v1/admin/accounts/login")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();

    if !status.is_success() {
        return Err(anyhow!("Failed with status: {}", status));
    }

    accounts.register_if_not_yet(&email, &password);

    // let body = res.json().await?;

    Ok(res)
}
