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
use serde_json::{json, Value};

pub async fn add_profile(accounts: &mut Accounts) -> Result<Response> {
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

    let res = login(&email, &password).await?;

    accounts.register_if_not_yet(&email, &password);

    Ok(res)
}

pub async fn login(email: &str, password: &str) -> Result<Response> {
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

    // Check if the status is a success.
    if status.is_success() {
        println!("Account successfully added to local config.");
    } else if status.is_client_error() {
        // Get the body of the response.
        let body = res.text().await?;
        return Err(anyhow!(
            "Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        // For other errors (like server errors)
        println!("An unexpected error occurred.");
    }

    Ok(res)
}

pub async fn get_jwt(response: Response) -> Result<String> {
    let body = response.text().await?;
    let json: Value =
        serde_json::from_str(&body).expect("Failed to parse JSON");
    let id_token = json["idToken"].as_str().expect("idToken is not a string");

    Ok(id_token.to_string())
}
