use console::style;
use std::{path::Path, sync::mpsc::Sender};
use terminal_link::Link;
use tokio::task::JoinSet;

use std::sync::mpsc::channel;
// use sui_framework_build::compiled_package::BuildConfig;
// use sui_json_rpc_types::{OwnedObjectRef, SuiObjectRead, SuiRawData};
// use sui_keys::keystore::AccountKeystore;
// use sui_sdk::{types::messages::Transaction, SuiClient};
// use sui_types::{
//     crypto::Signature, intent::Intent, messages::ExecuteTransactionRequestType,
// };

use crate::{
    collection_state::{CollectionState, ObjectType},
    consts::{NFT_PROTOCOL, VOLCANO_EMOJI},
    err::RustSdkError,
    utils::{get_active_address, get_client, get_keystore},
};

pub async fn publish_contract(
    package_dir: &Path,
    gas_budget: u64,
) -> Result<CollectionState, RustSdkError> {
    let client = get_client().await.unwrap();
    let keystore = get_keystore().await.unwrap();
    let active_address = get_active_address(&keystore).unwrap();

    println!("{} Compiling contract", style("WIP").cyan().bold());
    let compiled_modules_base_64 = BuildConfig::default()
        .build(package_dir.to_path_buf())?
        .get_package_base64(false);

    let compiled_modules = compiled_modules_base_64
        .into_iter()
        .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
        .collect::<Result<Vec<_>, _>>()?;

    println!("{} Compiling contract", style("DONE").green().bold());

    println!("{} Preparing transaction", style("WIP").cyan().bold());

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
    let mut signatures: Vec<Signature> = vec![];
    signatures.push(keystore.sign_secure(
        &active_address,
        &call,
        Intent::default(),
    )?);

    println!("{} Preparing transaction.", style("DONE").green().bold());

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );
    let tx =
        Transaction::from_data(call, Intent::default(), signatures).verify()?;

    let request_type =
        Some(ExecuteTransactionRequestType::WaitForLocalExecution);

    let response = client
        .quorum_driver()
        .execute_transaction(tx, request_type)
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

    // Creating a channel to send message with package ID
    let (sender, receiver) = channel();

    let objects_created = response.effects.unwrap().created;

    objects_created
        .iter()
        .map(|object| {
            // TODO: Remove this clone
            let object_ = object.clone();
            let client_ = client.clone();
            let sender_ = sender.clone();
            set.spawn(async move {
                print_object(&client_, &object_, sender_).await;
            });
        })
        .for_each(drop);

    let mut i = 1;
    while let Some(res) = set.join_next().await {
        res.unwrap();
        i += 1;
    }

    println!("A total of {} object have been created.", i);

    let mut collection_state = CollectionState::default();
    // It's three as we are interest in the MintCap, Collection and Package
    for _ in 0..3 {
        let object_type = receiver.recv().unwrap();
        match object_type {
            ObjectType::Package(_object_id) => {
                collection_state.contract = Some(object_type);
            }
            ObjectType::MintCap(_object_id) => {
                collection_state.mint_cap = Some(object_type);
            }
            ObjectType::Collection(_object_id) => {
                collection_state.collection = Some(object_type);
            }
            _ => {}
        }
    }

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network=devnet",
        collection_state.contract.as_ref().unwrap()
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(collection_state)
}

async fn print_object(
    client: &SuiClient,
    object: &OwnedObjectRef,
    tx: Sender<ObjectType>,
) {
    let object_id = object.reference.object_id;
    let object_read = client.read_api().get_object(object_id).await.unwrap();

    let mint_cap = format!("{}::mint_cap::MintCap", NFT_PROTOCOL);
    let collection = format!("{}::collection::Collection", NFT_PROTOCOL);

    if let SuiObjectRead::Exists(sui_object) = object_read {
        match sui_object.data {
            SuiRawData::MoveObject(raw_object) => {
                if raw_object.type_.contains(mint_cap.as_str()) {
                    println!("Mint Cap object ID: {}", object_id);
                    tx.send(ObjectType::MintCap(object_id)).unwrap();
                }
                if raw_object.type_.contains(collection.as_str()) {
                    println!("Collection object ID: {}", object_id);
                    tx.send(ObjectType::Collection(object_id)).unwrap();
                }
            }
            SuiRawData::Package(_raw_package) => {
                println!("Package object ID: {}", object_id);

                tx.send(ObjectType::Package(object_id)).unwrap();
            }
        }
    }
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}
