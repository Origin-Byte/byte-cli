use super::login;
use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::{anyhow, Result};
use console::style;
use dialoguer::{Input, Password};

pub async fn add_profile(accounts: &mut Accounts) -> Result<()> {
    let theme = get_dialoguer_theme();
    println!(
        "{}",
        style("Let's link your Suiplay account to the CLI.")
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

    let res = login(&email, &password).await?;

    let status = res.status();

    // Check if the status is a success.
    if status.is_success() {
        println!("We've successfully verified your account.");
    } else if status.is_client_error() {
        // Get the body of the response.
        let body = res.text().await?;
        return Err(anyhow!(
            "Error while verifying your account. Failed with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        // For other errors (like server errors)
        let body = res.text().await?;

        return Err(anyhow!(
            "Ups.. It seems that we've encountered a server side error {} with the following message: {}",
            status,
            body
        ));
    }

    accounts.register_if_not_yet(&email, &password);

    println!(
        "{}",
        style("Account is now lainked.").green().bold().on_bright()
    );

    Ok(())
}
