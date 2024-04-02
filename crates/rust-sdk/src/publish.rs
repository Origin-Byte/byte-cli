use anyhow::anyhow;
use console::style;
use std::env;
use std::path::Path;
use std::sync::mpsc::Sender;

use sui_json_rpc_types::SuiTransactionBlockResponse;
use sui_json_rpc_types::{OwnedObjectRef, SuiObjectDataOptions};
use sui_move_build::BuildConfig;
use sui_sdk::SuiClient;
use sui_types::base_types::{ObjectID, ObjectType, SuiAddress};
use sui_types::{
    base_types::ObjectRef,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::TransactionData,
};

use move_package::BuildConfig as MoveBuildConfig;
use sui_move::build::resolve_lock_file_path;

use crate::consts::{PRICE_PUBLISH, RECIPIENT_ADDRESS};
use crate::utils::{execute_tx, get_context};
use crate::{collection_state::ObjectType as OBObjectType, err::RustSdkError};
use std::str::FromStr;

/// Publishes a contract and returns the transaction response.
///
/// This function compiles the contract from the specified directory, creates a
/// transaction to publish it, and then executes the transaction.
///
/// # Arguments
/// * `sender` - The Sui address sending the transaction.
/// * `package_dir` - The directory path of the package to be published.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the transaction block response or a `RustSdkError`.
pub async fn publish_contract(
    sender: SuiAddress,
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();

    let data =
        prepare_publish_contract(sender, package_dir, gas_coin, gas_budget)
            .await?;

    println!("{} Preparing transaction.", style("DONE").green().bold());

    execute_tx(&wallet_ctx, data).await
}

/// Publishes a contract and performs a payment, returning the transaction
/// response.
///
/// Similar to `publish_contract`, but also includes a payment step in the
/// transaction.
///
/// # Arguments
/// * `package_dir` - The directory path of the package to be published.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the transaction block response or a `RustSdkError`.
pub async fn publish_contract_and_pay(
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<SuiTransactionBlockResponse, RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();
    let sender = wallet_ctx.config.active_address.unwrap();

    let data = prepare_publish_contract_and_pay(
        sender,
        package_dir,
        gas_coin,
        gas_budget,
    )
    .await?;

    println!("{} Preparing transaction.", style("DONE").green().bold());

    execute_tx(&wallet_ctx, data).await
}

/// Prepares transaction data for publishing a contract.
///
/// # Arguments
/// * `sender` - The Sui address sending the transaction.
/// * `package_dir` - The directory path of the package to be published.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the transaction data or a `RustSdkError`.
pub async fn prepare_publish_contract(
    sender: SuiAddress,
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<TransactionData, RustSdkError> {
    let builder = init_publish_pt(sender, package_dir).await?;

    let gas_coins = vec![gas_coin];

    Ok(TransactionData::new_programmable(
        sender,
        gas_coins, // Gas Objects
        builder.finish(),
        gas_budget, // Gas Budget
        1000,       // Gas Price
    ))
}

/// Prepares transaction data for publishing a contract and performing a
/// payment.
///
/// # Arguments
/// * `sender` - The Sui address sending the transaction.
/// * `package_dir` - The directory path of the package to be published.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the transaction data or a `RustSdkError`.
pub async fn prepare_publish_contract_and_pay(
    sender: SuiAddress,
    package_dir: &Path,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<TransactionData, RustSdkError> {
    let mut builder = init_publish_pt(sender, package_dir).await?;

    let ob_addr = SuiAddress::from_str(RECIPIENT_ADDRESS)?;
    builder.pay_sui(
        vec![ob_addr],       // recipients
        vec![PRICE_PUBLISH], // amounts
    )?;

    Ok(TransactionData::new_programmable(
        sender,
        vec![gas_coin], // Gas Objects
        builder.finish(),
        gas_budget, // Gas Budget
        1000,       // Gas Price
    ))
}

/// Initializes the programmable transaction builder for publishing a contract.
///
/// # Arguments
/// * `sender` - The Sui address sending the transaction.
/// * `package_dir` - The directory path of the package to be published.
///
/// # Returns
/// A result containing the initialized transaction builder or a `RustSdkError`.
async fn init_publish_pt(
    sender: SuiAddress,
    package_dir: &Path,
) -> Result<ProgrammableTransactionBuilder, RustSdkError> {
    let build_config = MoveBuildConfig::default();

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

    Ok(builder)
}

/// Retrieves the dependencies of the specified package.
///
/// # Arguments
/// * `build_config` - The build configuration for the Move package.
/// * `package_path` - The directory path of the package.
///
/// # Returns
/// A result containing a vector of object IDs representing dependencies or a
/// `RustSdkError`.
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

/// Prints information about an object.
///
/// This function retrieves the details of a specified object and sends its type
/// to a channel.
///
/// # Arguments
/// * `client` - The Sui client used to read object data.
/// * `object` - The reference to the owned object.
/// * `tx` - The sender channel used to transmit the object type.
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
