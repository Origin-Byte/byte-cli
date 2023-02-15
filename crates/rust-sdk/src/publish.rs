use console::{style, Emoji};
use std::path::Path;
use terminal_link::Link;
use tokio::task::JoinSet;

use sui_framework_build::compiled_package::BuildConfig;
use sui_json_rpc_types::{OwnedObjectRef, SuiObjectRead, SuiRawData};
use sui_keys::keystore::AccountKeystore;
use sui_sdk::{types::messages::Transaction, SuiClient};
use sui_types::{intent::Intent, messages::ExecuteTransactionRequestType};

use crate::{
    err::RustSdkError,
    utils::{get_client, get_keystore},
};

pub const VOLCANO_EMOJI: Emoji<'_, '_> = Emoji("🌋", "");
pub const NFT_PROTOCOL: &str = "0xeac7173b9977892adc10ee5d254bcb2498ec521f";

pub async fn publish_contract(
    package_dir: &Path,
    gas_budget: u64,
) -> Result<(), RustSdkError> {
    let client = get_client().await.unwrap();
    let keystore = get_keystore().await.unwrap();
    let active_address = keystore.addresses().first().cloned().unwrap();

    println!("{} Compiling contract.", style("WIP").cyan().bold());
    let compiled_modules_base_64 = BuildConfig::default()
        .build(package_dir.to_path_buf())?
        .get_package_base64(false);

    let compiled_modules = compiled_modules_base_64
        .into_iter()
        .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<_>, _>>()?;

    println!("{} Compiling contract.", style("DONE").green().bold());

    println!("{} Preparing transaction.", style("WIP").cyan().bold());
    let call = client
        .transaction_builder()
        .publish(
            active_address, // sender
            compiled_modules,
            None,       // Gas object, Node can pick one itself
            gas_budget, // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature =
        keystore.sign_secure(&active_address, &call, Intent::default())?;
    println!("{} Preparing transaction.", style("DONE").green().bold());

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );
    let response = client
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature)
                .verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await
        .unwrap();

    assert!(response.confirmed_local_execution);
    println!(
        "{} Sending and executing transaction.",
        style("Done").cyan().bold()
    );

    println!(
        "{} {}",
        VOLCANO_EMOJI,
        style("Contract has been successfuly deployed on-chain.")
            .green()
            .bold()
    );
    let mut set = JoinSet::new();

    let objects_created = response.effects.unwrap().created;

    objects_created
        .iter()
        .map(|object| {
            // TODO: Remove this clone
            let object_ = object.clone();
            let client_ = client.clone();
            set.spawn(async move {
                print_object(&client_, &object_).await;
            });
        })
        .for_each(drop);

    let mut i = 1;
    while let Some(res) = set.join_next().await {
        res.unwrap();
        i += 1;
    }

    println!("A total of {} object have been created.", i);

    let link =
        Link::new("Sui Explorer", "(https://explorer.sui.io?network=devnet");

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(())
}

async fn print_object(client: &SuiClient, object: &OwnedObjectRef) {
    let object_id = object.reference.object_id;
    let object_read = client.read_api().get_object(object_id).await.unwrap();

    let mint_cap = format!("{}::mint_cap::MintCap", NFT_PROTOCOL);
    let collection = format!("{}::collection::Collection", NFT_PROTOCOL);

    if let SuiObjectRead::Exists(sui_object) = object_read {
        match sui_object.data {
            SuiRawData::MoveObject(raw_object) => {
                if raw_object.type_.contains(mint_cap.as_str()) {
                    println!("Mint Cap object ID: {}", object_id);
                }
                if raw_object.type_.contains(collection.as_str()) {
                    println!("Collection object ID: {}", object_id);
                }
            }
            SuiRawData::Package(_raw_package) => {
                println!("Package object ID: {}", object_id);
            }
        }
    }
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}
