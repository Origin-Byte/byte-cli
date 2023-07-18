use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};
use reqwest::{Client, Response};
use serde_json::json;

pub async fn signup(accounts: &mut Accounts) -> Result<Response> {
    let theme = get_dialoguer_theme();
    println!(
        "{}",
        style("Welcome to SuiPlay! Let's create an account.")
            .blue()
            .bold()
            .dim()
    );

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
        .post("https://suiplay-api-1o7v724t.ew.gateway.dev/v1/admin/accounts/register")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();

    // Check if the status is a success.
    if status.is_success() {
        println!("You've been successfully registerd!");
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

    accounts.register_if_not_yet(&email, &password);

    accounts.set_main_if_none(email);

    Ok(res)
}
