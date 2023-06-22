use anyhow::anyhow;
use console::style;
use std::env;
use std::path::Path;
use std::sync::mpsc::Sender;
use sui_sdk::wallet_context::WalletContext;

use shared_crypto::intent::Intent;
use sui_json_rpc_types::{OwnedObjectRef, SuiObjectDataOptions};
use sui_json_rpc_types::{
    SuiTransactionBlockResponse,
};
use sui_keys::keystore::AccountKeystore;
use sui_move_build::BuildConfig;
use sui_sdk::{types::messages::Transaction, SuiClient};
use sui_types::base_types::{ObjectID, ObjectType, SuiAddress};
use sui_types::crypto::Signature;
use sui_types::{
    base_types::ObjectRef, messages::TransactionData,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
};

use move_package::BuildConfig as MoveBuildConfig;
use sui_move::build::resolve_lock_file_path;

use crate::consts::{PRICE_PUBLISH, RECIPIENT_ADDRESS};
use crate::utils::get_context;
use crate::{collection_state::ObjectType as OBObjectType, err::RustSdkError};
use std::str::FromStr;

pub async fn publish_contract(
    wallet_ctx: &WalletContext,
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let build_config = MoveBuildConfig::default();

    let context = get_context().await.unwrap();
    let keystore = &context.config.keystore;
    let sender = context.config.active_address.unwrap();

    println!("{} Compiling contract", style("WIP").cyan().bold());

    let (compiled_modules, dep_ids) = {
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

    let data = TransactionData::new_programmable(
        sender,
        vec![gas_coin], // Gas Objects
        builder.finish(),
        gas_budget, // Gas Budget
        1000,       // Gas Price
    );

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature =
        keystore.sign_secure(&sender, &data, Intent::sui_transaction())?;

    signatures.push(signature);

    println!("{} Preparing transaction.", style("DONE").green().bold());

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );

    let response = wallet_ctx
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    println!(
        "{} Sending and executing transaction.",
        style("Done").cyan().bold()
    );

    Ok(response)
}

pub async fn publish_contract_and_pay(
    wallet_ctx: &WalletContext,
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let build_config = MoveBuildConfig::default();

    let context = get_context().await.unwrap();
    let keystore = &context.config.keystore;
    let sender = context.config.active_address.unwrap();

    println!("{} Compiling contract", style("WIP").cyan().bold());

    let (compiled_modules, dep_ids) = {
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

    let ob_addr = SuiAddress::from_str(RECIPIENT_ADDRESS)?;

    builder.pay_sui(
        vec![ob_addr],       // recipients
        vec![PRICE_PUBLISH], // amounts
    )?;

    let data = TransactionData::new_programmable(
        sender,
        vec![gas_coin], // Gas Objects
        builder.finish(),
        gas_budget, // Gas Budget
        1000,       // Gas Price
    );

    // Sign transaction.
    let mut signatures: Vec<Signature> = vec![];

    let signature =
        keystore.sign_secure(&sender, &data, Intent::sui_transaction())?;

    signatures.push(signature);

    println!("{} Preparing transaction.", style("DONE").green().bold());

    // Execute the transaction.
    println!(
        "{} Sending and executing transaction.",
        style("WIN").cyan().bold()
    );

    let response = wallet_ctx
        .execute_transaction_block(
            Transaction::from_data(data, Intent::sui_transaction(), signatures)
                .verify()?,
        )
        .await?;

    println!(
        "{} Sending and executing transaction.",
        style("Done").cyan().bold()
    );

    Ok(response)
}

fn get_dependencies(
    build_config: &MoveBuildConfig,
    package_path: &Path,
) -> Result<Vec<ObjectID>, RustSdkError> {
    let cur_dir = env::current_dir()
        .map_err(|_| anyhow!(r#"This error should be unreachable"#))?;

    let build_config = resolve_lock_file_path(
        build_config.clone(),
        Some(package_path.to_path_buf()),
    )?;

    env::set_current_dir(cur_dir)
        .map_err(|_| anyhow!(r#"This error should be unreachable"#))?;

    let compiled_package = BuildConfig {
        config: build_config,
        run_bytecode_verifier: true,
        print_diags_to_stderr: true,
    }
    .build(package_path.to_path_buf())?;

    let deps = compiled_package
        .dependency_ids
        .published
        .iter()
        .map(|t| *t.1)
        .collect();

    Ok(deps)
}

pub async fn print_object(
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
