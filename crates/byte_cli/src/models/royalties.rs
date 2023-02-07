use super::{address_validator, number_validator, sender, FromPrompt};
use crate::prelude::get_dialoguer_theme;

use dialoguer::{Input, MultiSelect};
use gutenberg::models::royalties::{Royalties, Share};

const ROYALTY_OPTIONS: [&str; 2] = ["Proportional", "Constant", "None"];

impl FromPrompt for Share {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut address = Input::with_theme(&theme)
            .with_prompt("Who will receive the royalties")
            .default(sender().to_string())
            .validate_with(address_validator)
            .interact()
            .unwrap();

        if address.is_empty() {
            address.push_str(sender())
        }

        Ok(Share {
            address,
            // TODO: Support multiple shares
            share: 10000,
        })
    }
}

impl FromPrompt for Royalties {
    fn from_prompt() -> Result<Self, anyhow::Error>
    where
        Self: Sized,
    {
        let theme = get_dialoguer_theme();

        let mut policy = Royalties::default();

        for policy_index in MultiSelect::with_theme(&theme)
            .with_prompt("Which royalty policies do you want on your collection? (use [SPACEBAR] to select options you want and hit [ENTER] when done)")
            .items(&ROYALTY_OPTIONS)
            .interact()? {
                match ROYALTY_OPTIONS[policy_index] {
                    "Proportional" => {
                        let fee = Input::with_theme(&theme)
                            .with_prompt("What is the proportional royalty fee in basis points?")
                            .validate_with(number_validator)
                            .interact()?
                            .parse::<u64>()?;

                        policy.proportional = Some(fee);
                    },
                    "Constant" => {
                        let fee = Input::with_theme(&theme)
                            .with_prompt("What is the constant royalty fee in MIST?")
                            .validate_with(number_validator)
                            .interact()?
                            .parse::<u64>()?;

                        policy.constant = Some(fee);
                    }
                    _ => unreachable!()
                }
            };

        if policy.has_royalties() {
            // TODO: Support multiple shares
            let share = Share::from_prompt()?;
            policy.shares.push(share);
        }

        Ok(policy)
    }
}
