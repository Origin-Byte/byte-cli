use crate::{cli::get_dialoguer_theme, models::Accounts};
use anyhow::Result;
use console::style;
use dialoguer::Input;

pub async fn remove_profile(accounts: &mut Accounts) -> Result<()> {
    let theme = get_dialoguer_theme();

    let email: String = Input::with_theme(&theme)
        .with_prompt("E-mail:")
        .interact()
        .unwrap();

    accounts.remove_account(&email);
    println!("Successfully removed {}", email);

    let main_acc = accounts.get_main_account();

    if email == main_acc.email {
        if let Some(first_acc) = accounts.accounts.first() {
            println!("Setting account {} as default", first_acc.email);
            accounts.set_main(first_acc.email.clone());
        } else {
            println!("No accounts remaining");
            accounts.main = None;
        }
    }

    println!(
        "{}",
        style("Account is now unlinked.").green().bold().on_bright()
    );

    Ok(())
}
