use anyhow::{anyhow, Context, Result};
use console::style;
use gutenberg_types::Schema;
use reqwest::Client;
use rust_sdk::models::project::{
    AdminObjects, CollectionObjects, MintCap, Project,
};
use rust_sdk::utils::execute_tx;
use rust_sdk::{coin, consts::VOLCANO_EMOJI, utils::get_context};
use rust_sdk::{collection_state::ObjectType as OBObjectType, publish};
use serde_json::json;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;
use sui_sdk::rpc_types::{
    SuiTransactionBlockEffects, SuiTransactionBlockResponse,
};
use sui_sdk::types::transaction::{TransactionData, TransactionDataV1};
use terminal_link::Link;
use tokio::task::JoinSet;

use crate::models::Accounts;

use super::add_profile::{get_jwt, login};

pub fn parse_config(config_file: &Path) -> Result<Schema, anyhow::Error> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find configuration file "{}": {err}
Call `byte-cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Schema>(file).map_err(|err|anyhow!(r#"Could not parse configuration file "{}": {err}
Call `byte-cli init-collection-config to initialize the configuration file again."#, config_file.display()))
}

pub fn parse_state(config_file: &Path) -> Result<Project, anyhow::Error> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find state file "{}": {err}
Call `byte-cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Project>(file)
        .map_err(|err| anyhow!(r#"Failed to serialize project state: {err}."#))
}

// pub fn generate_contract(
//     schema: Schema,
//     output_dir: &Path,
// ) -> Result<(), anyhow::Error> {
//     println!("{} Generating contract", style("WIP").cyan().bold());

//     let package_name = schema.package_name();
//     let contract_dir = gutenberg::generate_contract_dir(&schema, output_dir);

//     // Write Move-main.toml
//     let flavours_path = contract_dir.join("flavours/");
//     fs::create_dir_all(&flavours_path).with_context(|| {
//         format!(
//             r#"Could not create "{path}""#,
//             path = flavours_path.display()
//         )
//     })?;

//     let main_toml_path = contract_dir.join("flavours/Move-main.toml");
//     let mut main_toml_file =
//         File::create(&main_toml_path).with_context(|| {
//             format!(
//                 r#"Could not create "{path}""#,
//                 path = main_toml_path.display()
//             )
//         })?;

//     main_toml_file
//         .write_all(
//             gutenberg::generate_manifest(package_name.clone())
//                 .to_string()?
//                 .as_bytes(),
//         )
//         .with_context(|| {
//             format!(
//                 r#"Could not write to {path}"#,
//                 path = main_toml_path.display()
//             )
//         })?;

//     // Write Move-test.toml
//     let test_toml_path = contract_dir.join("flavours/Move-test.toml");
//     let mut test_toml_file =
//         File::create(&test_toml_path).with_context(|| {
//             format!(
//                 r#"Could not create "{path}""#,
//                 path = main_toml_path.display()
//             )
//         })?;

//     test_toml_file
//         .write_all(
//             gutenberg::generate_manifest(package_name)
//                 .to_string()?
//                 .as_bytes(),
//         )
//         .with_context(|| {
//             format!(
//                 r#"Could not write to {path}"#,
//                 path = main_toml_path.display()
//             )
//         })?;

//     // Copy Main Move.toml
//     fs::copy(main_toml_path, contract_dir.join("Move.toml"))?;

//     // TODO: Implement license check
//     let is_demo = false;
//     gutenberg::generate_contract_with_schema(schema, is_demo)
//         .into_iter()
//         .for_each(|file| file.write_to_file(&contract_dir).unwrap());

//     println!("{} Generating contract", style("DONE").green().bold());

//     Ok(())
// }

pub async fn prepare_publish_contract(
    state: &mut Project,
    accounts: &Accounts,
    name: String,
    schema: &Schema,
    gas_budget: usize,
    // contract_dir: &Path,
    // ) -> Result<TransactionData, anyhow::Error> {
) -> Result<()> {
    let contract_dir = Path::new("suip/contract/");

    let wallet_ctx = rust_sdk::utils::get_context().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let client = Client::builder()
        .timeout(Duration::from_secs(150)) // Set a timeout of 30 seconds
        .build()?;

    let schema_json =
        serde_json::to_string(&schema).expect("Failed to serialize schema");

    // let schema_json_value: serde_json::Value =
    //     serde_json::from_str(&schema_json)?;

    let req_body = json!({
        "name": name,
        "configJson": schema_json,
        "sender": sender.to_string(),
        "gasBudget": gas_budget,
        "projectDir": contract_dir,
    });

    let main_acc = accounts.get_main_account();

    let login_result = login(&main_acc.email, &main_acc.password).await?;
    let jwt = get_jwt(login_result).await?;

    // println!("BODY: {}", req_body);

    let res = client
        .post("https://suiplay-api-1o7v724t.ew.gateway.dev/v1/admin/contracts/generateContractAndBuildPublishTransaction")
        // .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", jwt)) // Add the Authorization
        .json(&req_body)
        .send()
        .await?;

    print!("THE RESPONSE IS: {:?}", res);

    let status = res.status();

    // Check if the status is a success.
    if status.is_success() {
        println!("Successfully generated contract.");
    } else if status.is_client_error() {
        // Get the body of the response.
        let body = res.text().await?;
        return Err(anyhow!(
            "Client Error with status: {} and the following message: {}",
            status,
            body
        ));
    } else {
        let body = res.text().await?;
        return Err(anyhow!(
            "Server Error with status: {} and the following message: {:?}",
            status,
            body,
        ));
    }

    let body = res.text().await?;

    // println!("{}", body);

    let transaction_data_v1: TransactionDataV1 = serde_json::from_str(&body)?;
    let transaction_data = TransactionData::V1(transaction_data_v1);
    println!("{:?}", transaction_data);

    let response: SuiTransactionBlockResponse =
        execute_tx(&wallet_ctx, transaction_data).await?;

    println!("SUI RESPONSE {:?}", response);

    process_effects(state, response);

    Ok(())
}

// pub async fn publish_contract(
//     gas_budget: usize,
//     contract_dir: &Path,
// ) -> Result<SuiTransactionBlockResponse, anyhow::Error> {
//     let wallet_ctx = rust_sdk::utils::get_context().await?;
//     let client = wallet_ctx.get_client().await?;
//     let sender = wallet_ctx.config.active_address.unwrap();

//     let gas_coin = rust_sdk::utils::get_coin_ref(
//         &coin::get_max_coin(&client, sender).await?,
//     );

//     let response = publish::publish_contract_and_pay(
//         contract_dir,
//         gas_coin,
//         gas_budget as u64,
//     )
//     .await?;

//     Ok(response)
// }

pub async fn process_effects(
    state: &mut Project,
    response: SuiTransactionBlockResponse,
) -> Result<(), anyhow::Error> {
    let context = get_context().await.unwrap();
    let client = context.get_client().await?;

    println!(
        "{} {}",
        VOLCANO_EMOJI,
        style("Contract has been successfuly deployed on-chain.")
            .green()
            .bold()
    );
    let mut set = JoinSet::new();

    // Creating a channel to send message with package ID
    let (sender, receiver) = channel();

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    let objects_created = effects.created;

    objects_created
        .iter()
        .map(|object| {
            // TODO: Remove this clone
            let object_ = object.clone();
            let client_ = client.clone();
            let sender_ = sender.clone();
            set.spawn(async move {
                publish::print_object(&client_, &object_, sender_).await;
            });
        })
        .for_each(drop);

    let mut i = 1;
    while let Some(res) = set.join_next().await {
        res.unwrap();
        i += 1;
    }

    println!("A total of {} object have been created.", i);

    let mut j = 0;

    // It's three as we are interest in the MintCap, Collection and Package
    // We need to make sure we agree on the number of objects that are recorded
    while j < 3 {
        let object_type = receiver.recv().unwrap();
        match object_type {
            OBObjectType::Package(object_id) => {
                state.package_id = Some(object_id);
                j += 1;
            }
            OBObjectType::MintCap(object_id) => {
                let admin_objs =
                    state.admin_objects.get_or_insert(AdminObjects::empty());
                admin_objs.mint_caps.push(MintCap::new(object_id));
                j += 1;
            }
            OBObjectType::Collection(object_id) => {
                let col_objs = state
                    .collection_objects
                    .get_or_insert(CollectionObjects::empty());

                col_objs.collection = Some(object_id);

                j += 1;
            }
            _ => {}
        }
    }

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network=testnet",
        state.package_id.as_ref().unwrap()
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(())
}
