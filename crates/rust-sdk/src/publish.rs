use anyhow::anyhow;
use console::style;
use gag::Gag;
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use terminal_link::Link;
use tokio::task::JoinSet;

use shared_crypto::intent::Intent;
use std::sync::mpsc::channel;
use sui_json_rpc_types::SuiTransactionBlockEffects;
use sui_json_rpc_types::{OwnedObjectRef, SuiObjectDataOptions};
use sui_keys::keystore::AccountKeystore;
use sui_move_build::BuildConfig;
use sui_sdk::{types::messages::Transaction, SuiClient};
use sui_types::base_types::{ObjectID, ObjectType};
use sui_types::crypto::Signature;
use sui_types::{
    base_types::ObjectRef, messages::TransactionData,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};

use move_package::BuildConfig as MoveBuildConfig;
use sui_move::build::resolve_lock_file_path;

use crate::{
    collection_state::{CollectionState, ObjectType as OBObjectType},
    consts::VOLCANO_EMOJI,
    err::RustSdkError,
    utils::{get_active_address, get_client, get_context, get_keystore},
};

pub async fn publish_contract(
    package_dir: &PathBuf,
    gas_budget: u64,
) -> Result<CollectionState, RustSdkError> {
    let build_config = MoveBuildConfig::default();

    let context = get_context().await.unwrap();
    let client = get_client().await.unwrap();
    // Maybe get these from the context
    let keystore = get_keystore().await.unwrap();
    let sender = get_active_address(&keystore).unwrap();

    println!("{} Compiling contract", style("WIP").cyan().bold());

    let (compiled_modules, dep_ids) = {
        let _print_gag = Gag::stderr().unwrap();
        let compiled_modules_base_64 = BuildConfig::default();

        let compiled_modules_base_64 = compiled_modules_base_64
            .build(package_dir.to_path_buf())?
            .get_package_base64(false);

        let mods = compiled_modules_base_64
            .into_iter()
            .map(|data| data.to_vec().map_err(|e| anyhow::anyhow!(e)))
            .collect::<Result<Vec<_>, _>>()?;

        let dep_ids: Vec<ObjectID> =
            get_dependencies(&build_config, package_dir)?;

        (mods, dep_ids)
    };

    println!("{} Compiling contract", style("DONE").green().bold());
    println!("{} Preparing transaction", style("WIP").cyan().bold());

    let mut builder = ProgrammableTransactionBuilder::new();
    let upgrade_cap = builder.publish_upgradeable(compiled_modules, dep_ids);

    builder.transfer_arg(sender, upgrade_cap);

    let print_gag = Gag::stderr().unwrap();

    // Get gas object
    let coins: Vec<ObjectRef> = context
        .gas_objects(sender)
        .await?
        .iter()
        // Ok to unwrap() since `get_gas_objects` guarantees gas
        .map(|(_val, object)| {
            // let gas = GasCoin::try_from(object).unwrap();
            (object.object_id, object.version, object.digest)
        })
        .collect();

    let data = TransactionData::new_programmable(
        sender,
        coins, // Gas Objects
        builder.finish(),
        gas_budget, // Gas Budget
        1000,       // Gas Price
    );

    drop(print_gag);

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature = context.config.keystore.sign_secure(
        &sender,
        &data,
        Intent::sui_transaction(),
    )?;

    signatures.push(signature);

    println!("{} Preparing transaction.", style("DONE").green().bold());

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );

    let response = context
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

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

    let effects = match response.effects.unwrap() {
        SuiTransactionBlockEffects::V1(effects) => effects,
    };

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

    let mut j = 0;

    let mut collection_state = CollectionState::default();
    // It's three as we are interest in the MintCap, Collection and Package
    // We need to make sure we agree on the number of objects that are recorded
    while j < 3 {
        let object_type = receiver.recv().unwrap();
        match object_type {
            OBObjectType::Package(_object_id) => {
                collection_state.contract = Some(object_type);
                j += 1;
            }
            OBObjectType::MintCap(_object_id) => {
                collection_state.mint_cap = Some(object_type);
                j += 1;
            }
            OBObjectType::Collection(_object_id) => {
                collection_state.collection = Some(object_type);
                j += 1;
            }
            _ => {}
        }
    }

    let explorer_link = format!(
        "https://explorer.sui.io/object/{}?network=testnet",
        collection_state.contract.as_ref().unwrap()
    );

    let link = Link::new("Sui Explorer", explorer_link.as_str());

    println!(
        "You can now find your collection package on the {}",
        style(link).blue().bold().underlined(),
    );

    Ok(collection_state)
}

fn get_dependencies(
    build_config: &MoveBuildConfig,
    package_path: &PathBuf,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let cur_dir = env::current_dir()
        .map_err(|_| anyhow!(r#"This error should be unreachable"#))?;

    let build_config = resolve_lock_file_path(
        build_config.clone(),
        Some(package_path.clone()),
    )?;

    env::set_current_dir(&cur_dir)
        .map_err(|_| anyhow!(r#"This error should be unreachable"#))?;

    let compiled_package = BuildConfig {
        config: build_config,
        run_bytecode_verifier: true,
        print_diags_to_stderr: true,
    }
    .build(package_path.clone())?;

    let deps = compiled_package
        .dependency_ids
        .published
        .iter()
        .map(|t| *t.1)
        .collect();

    Ok(deps)
}

async fn print_object(
    client: &SuiClient,
    object: &OwnedObjectRef,
    tx: Sender<OBObjectType>,
) {
    let object_id = object.reference.object_id;

    // get_owned_objects
    let object_read = client
        .read_api()
        .get_object_with_options(
            object_id,
            SuiObjectDataOptions::full_content(),
        )
        .await
        .unwrap();

    // TODO: Add Publisher + UpgradeCap

    if let Some(object_data) = object_read.data {
        let obj_type = object_data.type_.unwrap();

        match obj_type {
            ObjectType::Struct(object_type) => {
                if object_type.name().as_str() == "MintCap" {
                    println!("Mint Cap: {}", object_id);
                    tx.send(OBObjectType::MintCap(object_id)).unwrap();
                }
                if object_type.name().as_str() == "Collection" {
                    println!("Collection: {}", object_id);
                    tx.send(OBObjectType::Collection(object_id)).unwrap();
                }
                if object_type.name().as_str() == "BpsRoyaltyStrategy" {
                    println!("BpsRoyaltyStrategy: {}", object_id);
                    tx.send(OBObjectType::BpsRoyaltyStrategy(object_id))
                        .unwrap();
                }
                if object_type.name().as_str() == "PolicyCap" {
                    println!("PolicyCap: {}", object_id);
                    tx.send(OBObjectType::PolicyCap(object_id)).unwrap();
                }
                if object_type.name().as_str() == "Policy" {
                    // println!("{:?}", yooooh);
                    println!("Policy: {}", object_id);
                    tx.send(OBObjectType::Policy(object_id)).unwrap();
                }
                if object_type.name().as_str() == "TransferPolicy" {
                    println!("TransferPolicy: {}", object_id);
                    tx.send(OBObjectType::TransferPolicy(object_id)).unwrap();
                }
                if object_type.name().as_str() == "TransferPolicyCap" {
                    println!("TransferPolicyCap: {}", object_id);
                    tx.send(OBObjectType::TransferPolicyCap(object_id))
                        .unwrap();
                }
            }
            ObjectType::Package => {
                println!("Package object ID: {}", object_data.object_id);
                tx.send(OBObjectType::Package(object_id)).unwrap();
            }
        }
    }
}

pub enum SuiArgType {
    StringSlice,
    ObjectId,
}
