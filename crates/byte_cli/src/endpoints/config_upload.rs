use crate::models::FromPrompt;
use crate::prelude::*;
use anyhow::anyhow;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Confirm, Input, MultiSelect, Password, Select};

use gutenberg::{
    models::{
        marketplace::{Listings, Marketplace},
        nft,
        royalties::Royalties,
        tags::Tags,
    },
    storage::{self, aws, nft_storage, pinata, Storage},
    Schema,
};

use serde::Serialize;
use std::fs::File;
use std::path::{Path, PathBuf};

const STORAGE_OPTIONS: [&str; 5] =
    ["AWS", "Pinata", "NftStorage", "Bundlr", "SHDW"];

pub fn init_upload_config(path: &str) -> Result<Schema, anyhow::Error> {
    let mut schema = super::try_read_config(path)?;

    let theme = get_dialoguer_theme();

    let number_validator = |input: &String| -> Result<(), String> {
        if input.parse::<u64>().is_err() {
            Err(format!("Couldn't parse input of '{}' to a number.", input))
        } else {
            Ok(())
        }
    };

    let storage_type = select(
        &theme,
        "Which storage service would you like to use?",
        &STORAGE_OPTIONS,
    )?;

    match storage_type {
        "AWS" => {
            // The best would be to fetch all the profiles for you and use this as multiselect
            let profile = Input::with_theme(&theme)
                .with_prompt("What is the name of your AWS profile?")
                .default(String::from("default"))
                .interact()
                .unwrap();

            let region = Input::with_theme(&theme)
                .with_prompt("Which AWS region? (leave blank to default to profile region)")
                .default(String::from("default"))
                .interact()
                .unwrap();

            // TODO
            // if region == "default" {}

            let bucket = Input::with_theme(&theme)
                .with_prompt("What is the name of the S3 bucket?")
                .interact()
                .unwrap();

            let directory = Input::with_theme(&theme)
                .with_prompt("Do you want to upload the assets to a specific directory? If so, what's the directory name? (Leave blank to default to the bucket root)")
                .default(String::from(""))
                .interact()
                .unwrap();

            let config =
                aws::AWSConfig::new(bucket, directory, region, profile);

            schema.storage = Storage::Aws(config);

            Ok(schema)
        }
        "Pinata" => {
            let jwt = Password::with_theme(&theme)
                .with_prompt("What is your Pinata JWT? (Keep this a secret")
                .interact()
                .unwrap();

            let upload_gateway = Input::with_theme(&theme)
                .with_prompt("Which gateway API will you use for upload? (Click enter for default gateway)")
                .default(String::from("https://api.pinata.cloud"))
                .interact()
                .unwrap();

            // TODO: Where is this used?
            let _retrieval_gateway = Input::with_theme(&theme)
                .with_prompt("Which gateway API will you use for retrieval? (Click enter for default gateway)")
                .default(String::from("https://gateway.pinata.cloud"))
                .interact()
                .unwrap();

            let parallel_limit =
                Input::with_theme(&theme)
                    .with_prompt("What is the limit of concurrent uploads? (Click enter for the default limit)")
                    .default(String::from("45"))
                    .validate_with(number_validator)
                    .interact()
                    .unwrap()
                    .parse::<u16>()
                    .expect("Failed to parse String into u64 - This error should not occur has input has been already validated.");

            let config =
                pinata::PinataConfig::new(jwt, upload_gateway, parallel_limit);

            schema.storage = Storage::Pinata(config);

            Ok(schema)
        }
        "NftStorage" => {
            let auth_token = Input::with_theme(&theme)
                .with_prompt("What is the authentication token=")
                .interact()
                .unwrap();

            let config = nft_storage::NftStorageConfig::new(auth_token);

            schema.storage = Storage::NftStorage(config);

            Ok(schema)
        }
        // "Bundlr" => Ok(()),
        // "SHDW" => Ok(()),
        _ => Err(anyhow!(
            "Unsupported Storage Type. This error should not occur"
        )),
    }
}

pub fn select<'a>(
    theme: &ColorfulTheme,
    prompt: &str,
    options: &[&'a str],
) -> anyhow::Result<&'a str> {
    let index = Select::with_theme(theme)
        .with_prompt(prompt)
        .items(options)
        .interact()
        .unwrap();

    Ok(options[index])
}
