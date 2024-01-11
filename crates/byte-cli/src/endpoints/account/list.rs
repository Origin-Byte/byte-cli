use crate::models::Accounts;
use anyhow::Result;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

/// Lists all accounts.
///
/// # Arguments
/// * `accounts` - A reference to the Accounts struct.
///
/// # Returns
/// Result type which is either empty (Ok) or contains an error (Err).
pub fn list(accounts: &Accounts) -> Result<()> {
    // Extracts emails from the accounts and collects them into a vector.
    let emails = accounts
        .accounts
        .iter()
        .map(|acc| acc.email.as_str())
        .collect::<Vec<&str>>();

    // Wraps the email list in the AccountList struct.
    let list = AccountList(emails);

    // Prints the list of accounts.
    println!("Accounts:");
    println!("{}", list);

    Ok(())
}

/// A struct to hold and display a list of account emails.
pub struct AccountList<'a>(Vec<&'a str>);

/// Implements the Display trait for AccountList for custom formatting.
impl<'a> Display for AccountList<'a> {
    /// Formats the account list for display.
    ///
    /// # Arguments
    /// * `f` - A mutable reference to a Formatter.
    ///
    /// # Returns
    /// A Result that is Ok if formatting succeeds, or an Err otherwise.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();

        // Iterates over accounts and writes each to the writer string.
        for account in &self.0 {
            writeln!(writer, " {}", account)?;
        }

        // Writes the formatted string to the formatter, trimming trailing newlines.
        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}
