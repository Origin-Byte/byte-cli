use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};
use reqwest::Client;
use serde_json::{json, Value};

/// Asynchronously signs up a user.
///
/// # Arguments
/// * `accounts` - A mutable reference to the Accounts struct.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
pub async fn signup(accounts: &mut Accounts) -> Result<()> {
    // Retrieves a dialoguer theme for consistent styling.
    let theme = get_dialoguer_theme();

    // Prints a welcome message using styled console output.
    println!(
        "{}",
        style("Welcome to SuiPlay! Let's create an account.")
            .green()
            .bold()
            .on_bright()
    );

    // Collects the user's email using an input field with a predefined theme.
    let email: String = Input::with_theme(&theme)
        .with_prompt("E-mail:")
        .interact()
        .unwrap();

    // Collects the user's password securely.
    let password = Password::with_theme(&theme)
        .with_prompt("Password")
        .interact()
        .unwrap();

    // Creates a new HTTP client instance.
    let client = Client::new();

    // Prepares the request body in JSON format.
    let req_body = json!({
        "email": email,
        "password": password,
    });

    // Sends a POST request to the registration endpoint.
    let res = client
        .post("https://suiplay-api.originbyte.io/v1/admin/accounts/register")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req_body)
        .send()
        .await?;

    // Extracts the status and body from the response.
    let status = res.status();
    let body = res.text().await?;

    // Error handling based on the status of the response.
    if status.is_success() {
        println!("You've been successfully registered!");
    } else if status.is_client_error() {
        return Err(anyhow!(
            "Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        return Err(anyhow!(
            "Ups.. It seems that we've encountered a server side error {} with the following message: {}",
            status,
            body
        ));
    }

    // Registers the user in the local account manager.
    accounts.register_if_not_yet(&email, &password);
    // Sets the newly registered account as the main account if none exists.
    accounts.set_main_if_none(email);

    // Parses the JSON response body.
    let json_value: Value = serde_json::from_str(&body).expect("Failed to parse JSON");

    // Extracts and prints the message from the JSON response.
    let message = json_value["message"]
        .as_str()
        .expect("Failed to parse field `message` from the body.");

    println!(
        "{}",
        style(format!("{}", message)).green().bold().on_bright()
    );

    Ok(())
}
