use crate::models::Accounts;
use anyhow::Result;
use std::fmt::Write;
use std::fmt::{Display, Formatter};

pub fn list(accounts: &Accounts) -> Result<()> {
    let emails = accounts
        .accounts
        .iter()
        .map(|acc| acc.email.as_str())
        .collect::<Vec<&str>>();

    let list = AccountList(emails);

    println!("Accounts:");
    println!("{}", list);

    Ok(())
}

pub struct AccountList<'a>(Vec<&'a str>);

impl<'a> Display for AccountList<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut writer = String::new();

        for account in &self.0 {
            writeln!(writer, " {}", account)?;
        }

        write!(f, "{}", writer.trim_end_matches('\n'))
    }
}
