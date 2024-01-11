use super::login;
use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};

/// Asynchronously adds a profile to the account.
///
/// # Arguments
/// * `accounts` - A mutable reference to the Accounts struct.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
pub async fn add_profile(accounts: &mut Accounts) -> Result<()> {
    // Retrieves a dialoguer theme for consistent styling.
    let theme = get_dialoguer_theme();

    // Prints a message indicating the process of linking an account.
    println!(
        "{}",
        style("Let's link your Suiplay account to the CLI.")
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

    // Calls the login function with the provided email and password.
    let res = login(&email, &password).await?;

    // Extracts the status from the login response.
    let status = res.status();

    // Error handling based on the status of the response.
    if status.is_success() {
        println!("We've successfully verified your account.");
    } else if status.is_client_error() {
        // Fetches and returns an error message for client-side errors.
        let body = res.text().await?;
        return Err(anyhow!(
            "Error while verifying your account. Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        // Fetches and returns an error message for server-side errors.
        let body = res.text().await?;

        return Err(anyhow!(
            "Ups.. It seems that we've encountered a server side error {} with the following message: {}",
            status,
            body
        ));
    }

    // Registers the user in the local account manager if not already registered.
    accounts.register_if_not_yet(&email, &password);

    // Prints a confirmation message indicating successful account linking.
    println!(
        "{}",
        style("Account is now linked.").green().bold().on_bright()
    );

    Ok(())
}
