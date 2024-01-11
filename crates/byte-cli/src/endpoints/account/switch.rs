use crate::models::Accounts;
use anyhow::{anyhow, Result};

/// Switches the main account to the specified email account.
///
/// # Arguments
/// * `email` - The email of the account to switch to.
/// * `accounts` - A mutable reference to the Accounts struct.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
///
/// # Errors
/// Returns an error if the email account is not found in the local storage.
pub fn switch(email: String, accounts: &mut Accounts) -> Result<()> {
    // Checks if the given email is registered in the accounts.
    if !accounts.check_if_registered(&email) {
        // Returns an error if the email is not registered.
        return Err(anyhow!("Email account not found in local storage. Run `byte account link` to link your SuiPlay account or `byte account create` to create a new one."));
    }

    // Sets the given email as the main account.
    accounts.set_main(email);

    Ok(())
}
