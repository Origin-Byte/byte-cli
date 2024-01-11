/// Imports necessary modules and structs.
use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::Result;
use console::style;
use dialoguer::Input;

/// Asynchronously removes a profile from the accounts.
///
/// # Arguments
/// * `accounts` - A mutable reference to the Accounts struct.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
///
/// # Functionality
/// - Prompts the user to enter the email of the account they wish to remove.
/// - Removes the specified account from the list of accounts.
/// - If the removed account is the main account, sets another account as the main account, if available.
/// - Prints a confirmation message upon successful removal.
pub async fn remove_profile(accounts: &mut Accounts) -> Result<()> {
    // Retrieves a dialoguer theme for consistent styling.
    let theme = get_dialoguer_theme();

    // Collects the email of the account to be removed.
    let email: String = Input::with_theme(&theme)
        .with_prompt("E-mail:")
        .interact()
        .unwrap();

    // Removes the account associated with the provided email.
    accounts.remove_account(&email);
    println!("Successfully removed {}", email);

    // Retrieves the main account.
    let main_acc = accounts.get_main_account();

    // Checks if the removed account was the main account.
    if email == main_acc.email {
        // Sets another account as the main account if available.
        if let Some(first_acc) = accounts.accounts.first() {
            println!("Setting account {} as default", first_acc.email);
            accounts.set_main(first_acc.email.clone());
        } else {
            // Notifies that no accounts are remaining if the list is empty.
            println!("No accounts remaining");
            accounts.main = None;
        }
    }

    // Prints a confirmation message about the account being unlinked.
    println!(
        "{}",
        style("Account is now unlinked.").green().bold().on_bright()
    );

    Ok(())
}
