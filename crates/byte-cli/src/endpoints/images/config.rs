use crate::cli::get_dialoguer_theme;
use anyhow::anyhow;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Password, Select};
use uploader::{
    storage::{aws, pinata},
    writer::Storage,
};

/// Storage options for uploading.
const STORAGE_OPTIONS: [&str; 2] = ["AWS", "Pinata"];

/// Initializes and returns the configuration for a chosen storage service.
///
/// # Returns
/// Result containing the Storage configuration or an error.
///
/// # Functionality
/// - Prompts the user to select a storage service (AWS or Pinata).
/// - Collects necessary configuration details based on the selected service.
/// - Validates input where necessary (e.g., numerical values).
/// - Constructs and returns the appropriate Storage configuration.
pub fn init_upload_config() -> Result<Storage, anyhow::Error> {
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
                aws::AWSConfig::new(bucket, directory, region, profile)?;

            Ok(Storage::Aws(config))
        }
        "Pinata" => {
            let jwt = Password::with_theme(&theme)
                .with_prompt("What is your Pinata JWT? (Keep this a secret)")
                .interact()
                .unwrap();

            let upload_gateway = Input::with_theme(&theme)
                .with_prompt("Which gateway API will you use for upload? (Click enter for default gateway)")
                .default(String::from("https://api.pinata.cloud/"))
                .interact()
                .unwrap();

            let retrieval_gateway = Input::with_theme(&theme)
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

            let config = pinata::PinataConfig::new(
                jwt,
                upload_gateway,
                retrieval_gateway,
                parallel_limit,
            );

            Ok(Storage::Pinata(config))
        }
        // TODO: Add back
        // "NftStorage" => {
        //     let auth_token = Input::with_theme(&theme)
        //         .with_prompt("What is the authentication token?")
        //         .interact()
        //         .unwrap();

        //     let config = nft_storage::NftStorageConfig::new(auth_token);

        //     Ok(Storage::NftStorage(config))
        // }
        _ => Err(anyhow!(
            "Unsupported Storage Type. This error should not occur"
        )),
    }
}

/// Helper function to display a selection prompt and return the chosen option.
///
/// # Arguments
/// * `theme` - A reference to a ColorfulTheme for styling the prompt.
/// * `prompt` - The prompt message to display.
/// * `options` - An array of options to present to the user.
///
/// # Returns
/// Result containing the selected option or an error.
///
/// # Functionality
/// - Displays a selection prompt with the provided options.
/// - Returns the option chosen by the user.
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
