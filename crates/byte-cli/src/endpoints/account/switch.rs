use crate::models::Accounts;
use anyhow::{anyhow, Result};

pub fn switch(email: String, accounts: &mut Accounts) -> Result<()> {
    if !accounts.check_if_registered(&email) {
        return Err(anyhow!("Email account not found in local storage. Run `byte account link` to link your SuiPlay account or `byte account create` to create a new one."));
    }
    accounts.set_main(email);

    Ok(())
}
