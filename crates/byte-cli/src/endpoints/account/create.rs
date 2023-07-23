use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};
use reqwest::Client;
use serde_json::{json, Value};

pub async fn signup(accounts: &mut Accounts) -> Result<()> {
    let theme = get_dialoguer_theme();

    println!(
        "{}",
        style("Welcome to SuiPlay! Let's create an account.")
            .green()
            .bold()
            .on_bright()
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
        .post("https://suiplay-api.originbyte.io/v1/admin/accounts/register")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    let status = res.status();
    let body = res.text().await?;

    // Check if the status is a success.
    if status.is_success() {
        println!("You've been successfully registerd!");
    } else if status.is_client_error() {
        // Get the body of the response.
        return Err(anyhow!(
            "Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        // For other errors (like server errors)
        return Err(anyhow!(
            "Ups.. It seems that we've encountered a server side error {} with the following message: {}",
            status,
            body
        ));
    }

    accounts.register_if_not_yet(&email, &password);
    accounts.set_main_if_none(email);

    // Parse the JSON string into a serde_json::Value object
    let json_value: Value =
        serde_json::from_str(&body).expect("Failed to parse JSON");

    let message = json_value["message"]
        .as_str()
        .expect("Failed to parse field `message` from the body.");

    println!(
        "{}",
        style(format!("{}", message)).green().bold().on_bright()
    );

    Ok(())
}
