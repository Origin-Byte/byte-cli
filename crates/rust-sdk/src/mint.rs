use crate::{
    err::{self, RustSdkError},
    metadata::Metadata,
    utils::{
        execute_tx, gas_objects, get_context, get_reference_gas_price,
        CloneClient, MoveType,
    },
};
use anyhow::Result;
use move_core_types::identifier::Identifier;
use std::{ops::Deref, str::FromStr, sync::Arc};
use sui_json_rpc_types::{SuiExecutionStatus, SuiTransactionBlockResponse};
use sui_json_rpc_types::{SuiObjectDataOptions, SuiTransactionBlockEffects};
use sui_sdk::{
    types::base_types::{ObjectID, SuiAddress},
    wallet_context::WalletContext,
    SuiClient,
};
use sui_types::{
    base_types::ObjectRef,
    object::Owner,
    parse_sui_type_tag,
    programmable_transaction_builder::ProgrammableTransactionBuilder,
    transaction::{CallArg, ObjectArg, TransactionData},
};
use tokio::task::JoinHandle;

/// Enum representing the possible outcomes of a mint operation.
///
/// # Variants
/// * `Success(Vec<String>)` - Indicates a successful mint operation with a list
///   of object IDs created.
/// * `Error(String)` - Represents an error that occurred during the minting
///   process, along with an error message.
pub enum MintEffect {
    Success(Vec<String>),
    Error(String),
}

/// Enum defining various functions for minting.
///
/// # Variants
/// * `WalletAirdrop` - Represents a wallet airdrop function.
/// * `KioskAirdrop(ObjectRef)` - Represents a kiosk airdrop function with the
///   object reference.
/// * `Warehouse(ObjectRef)` - Represents a warehouse function with the object
///   reference.
pub enum MintFunction {
    WalletAirdrop,
    KioskAirdrop(ObjectRef),
    Warehouse(ObjectRef),
}

/// Asynchronously creates a warehouse for managing collections.
///
/// # Arguments
/// * `collection_type` - The type of the collection.
/// * `package_id` - The ID of the package.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the ObjectID of the created warehouse or an error.
pub async fn create_warehouse(
    collection_type: MoveType,
    package_id: ObjectID,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<ObjectID, RustSdkError> {
    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await?;
    let sender = wallet_ctx.config.active_address.unwrap();

    let data = prepare_create_warehouse(
        &client,
        sender,
        collection_type,
        package_id,
        gas_coin,
        gas_budget,
    )
    .await?;

    let response = execute_tx(&wallet_ctx, data).await?;

    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    assert!(effects.status.is_ok());

    let warehouse_id = effects.created.first().unwrap().reference.object_id;

    Ok(warehouse_id)
}

/// Prepares transaction data for creating a warehouse.
///
/// # Arguments
/// * `client` - Client interface for interacting with Sui.
/// * `sender` - The address sending the transaction.
/// * `collection_type` - The type of the collection.
/// * `package_id` - The ID of the package.
/// * `gas_coin` - The gas coin object reference.
/// * `gas_budget` - The gas budget for the transaction.
///
/// # Returns
/// A result containing the transaction data or an error.
pub async fn prepare_create_warehouse(
    client: impl Deref<Target = SuiClient>,
    sender: SuiAddress,
    collection_type: MoveType,
    package_id: ObjectID,
    gas_coin: ObjectRef,
    gas_budget: u64,
) -> Result<TransactionData, RustSdkError> {
    let gas_price = get_reference_gas_price(client).await?;

    let collection_type = collection_type.write_type();
    let module = Identifier::from_str("warehouse")?;
    let function = Identifier::from_str("init_warehouse")?;

    let mut builder = ProgrammableTransactionBuilder::new();
    let _res = builder.move_call(
        package_id,                                          // Package ID
        module,                                              // Module Name
        function,                                            // Function Name
        vec![parse_sui_type_tag(collection_type.as_str())?], // Type Arguments
        vec![],                                              // Call Arguments
    );

    Ok(TransactionData::new_programmable(
        sender,
        vec![gas_coin], // Gas Objects
        builder.finish(),
        gas_budget,
        gas_price,
    ))
}

/// Handles the asynchronous minting of NFTs to a warehouse.
///
/// # Arguments
/// * `data` - Data for minting NFTs.
/// * `wallet_ctx` - Context of the wallet.
/// * `package_id` - The ID of the package.
/// * `module_name` - The name of the module.
/// * `gas_budget` - The gas budget for the transaction.
/// * `gas_coin` - Optional gas coin object reference.
/// * `sender` - The address sending the transaction.
/// * `warehouse` - The warehouse where NFTs will be minted.
/// * `mint_cap` - The mint capability object reference.
///
/// # Returns
/// A join handle for the asynchronous task.
#[allow(clippy::too_many_arguments)]
pub async fn handle_mint_nfts_to_warehouse(
    data: Vec<(u32, Metadata)>,
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: u64,
    gas_coin: Option<Arc<ObjectRef>>,
    sender: SuiAddress,
    warehouse: Arc<String>,
    mint_cap: Arc<String>,
) -> JoinHandle<Result<MintEffect, RustSdkError>> {
    tokio::spawn(async move {
        mint_nfts_to_warehouse(
            data,
            wallet_ctx,
            package_id,
            module_name,
            gas_budget,
            gas_coin,
            sender,
            warehouse,
            mint_cap,
        )
        .await
    })
}

/// Asynchronously mints NFTs to a specified warehouse.
///
/// This function manages the process of minting NFTs, including transaction
/// preparation and execution.
///
/// # Arguments
/// * `data` - A vector of tuples containing quantity and metadata for each NFT.
/// * `wallet_ctx` - The wallet context for transaction signing and management.
/// * `package_id` - The ID of the package containing the minting logic.
/// * `module_name` - The name of the module within the package.
/// * `gas_budget` - The gas budget for the transaction.
/// * `gas_coin` - Optional object reference for the gas coin.
/// * `sender` - The SuiAddress initiating the transaction.
/// * `warehouse` - The ID of the warehouse where NFTs will be minted.
/// * `mint_cap` - The mint capability object reference.
///
/// # Returns
/// A result containing the minting effect (success or error) or a
/// `RustSdkError`.
#[allow(clippy::too_many_arguments)]
pub async fn mint_nfts_to_warehouse(
    data: Vec<(u32, Metadata)>,
    wallet_ctx: Arc<WalletContext>,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: u64,
    gas_coin: Option<Arc<ObjectRef>>,
    sender: SuiAddress,
    warehouse: Arc<String>,
    mint_cap: Arc<String>,
) -> Result<MintEffect, RustSdkError> {
    let client = wallet_ctx.get_client().await?;

    let data = prepare_mint_nfts_to_warehouse(
        data,
        &client, // TODO: Should this be Arc instead?
        package_id,
        module_name,
        gas_budget,
        gas_coin,
        sender,
        warehouse.clone(),
        mint_cap,
    )
    .await?;

    // Execute the transaction.
    let response = execute_tx(wallet_ctx, data).await?;

    Ok(handle_mint_effects(response, warehouse)?)
}

/// Prepares the transaction data for minting NFTs to a warehouse.
///
/// This function creates the transaction data required for minting NFTs,
/// including the necessary arguments and gas settings.
///
/// # Type Parameters
/// * `C` - The client type, bounded by `Deref` targeting `SuiClient` and
///   `CloneClient`.
///
/// # Arguments
/// * `data` - A vector of tuples containing quantity and metadata for each NFT.
/// * `client` - The client used for retrieving network-related information.
/// * `package_id`, `module_name`, `gas_budget`, `gas_coin`, `sender`,
///   `warehouse`, `mint_cap` - Parameters for the minting transaction.
///
/// # Returns
/// A result containing the prepared transaction data or a `RustSdkError`.
#[allow(clippy::too_many_arguments)]
pub async fn prepare_mint_nfts_to_warehouse<C>(
    mut data: Vec<(u32, Metadata)>,
    client: C,
    package_id: Arc<String>,
    module_name: Arc<String>,
    gas_budget: u64,
    gas_coin: Option<Arc<ObjectRef>>,
    sender: SuiAddress,
    warehouse: Arc<String>,
    mint_cap: Arc<String>,
) -> Result<TransactionData, RustSdkError>
where
    C: Deref<Target = SuiClient> + CloneClient,
{
    let package_id = ObjectID::from_str(package_id.as_str())
        .map_err(|err| err::object_id(err, package_id.as_str()))?;
    let warehouse_id = ObjectID::from_str(warehouse.as_str())
        .map_err(|err| err::object_id(err, warehouse.as_str()))?;
    let mint_cap_id = ObjectID::from_str(mint_cap.as_str())
        .map_err(|err| err::object_id(err, mint_cap.as_str()))?;

    let gas_price = get_reference_gas_price(client.clone_client()).await?;

    let mut builder = ProgrammableTransactionBuilder::new();

    let objs = client
        .clone_client()
        .read_api()
        .multi_get_object_with_options(
            vec![mint_cap_id, warehouse_id],
            SuiObjectDataOptions::full_content(),
        )
        .await
        .unwrap();

    // Iterate over the entries and consume them
    while let Some((_index, nft_data)) = data.pop() {
        let mut args = nft_data.into_args()?;
        objs.iter().for_each(|obj| {
            let obj_data = obj.data.as_ref().unwrap();
            let obj_ref: ObjectRef =
                (obj_data.object_id, obj_data.version, obj_data.digest);
            args.push(CallArg::Object(ObjectArg::ImmOrOwnedObject(obj_ref)));
        });

        builder.move_call(
            package_id,
            Identifier::new(module_name.as_str()).unwrap(),
            Identifier::new("mint_nft_to_warehouse").unwrap(),
            vec![],
            args,
        )?;
    }

    // Get gas object
    let coins: Vec<ObjectRef> = match gas_coin {
        Some(coin) => vec![*coin],
        None => {
            gas_objects(client, sender)
                .await?
                .iter()
                // Ok to unwrap() since `get_gas_objects` guarantees gas
                .map(|(_val, object)| {
                    (object.object_id, object.version, object.digest)
                })
                .collect()
        }
    };

    let pt = builder.finish();

    Ok(TransactionData::new_programmable(
        sender, coins, pt, gas_budget, gas_price,
    ))
}

/// Handles the effects of a mint transaction.
///
/// This function processes the response of a mint transaction, extracting the
/// results or the error based on the transaction status.
///
/// # Arguments
/// * `response` - The transaction block response from executing the mint
///   transaction.
/// * `warehouse_id` - The ID of the warehouse where NFTs are minted.
///
/// # Returns
/// A result containing the mint effect (success or error) or a `RustSdkError`.
pub fn handle_mint_effects(
    response: SuiTransactionBlockResponse,
    warehouse_id: Arc<String>,
) -> Result<MintEffect, RustSdkError> {
    let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

    match effects.status {
        SuiExecutionStatus::Success => {
            let obj_ids = effects.created;
            let warehouse_addr =
                SuiAddress::from_str(warehouse_id.as_str()).unwrap();

            let nfts = obj_ids
                .iter()
                .filter_map(|obj_ref| {
                    // Filters out dynamic fields
                    if let Owner::ObjectOwner(owner) = obj_ref.owner {
                        if owner == warehouse_addr {
                            Some(obj_ref.reference.object_id.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<String>>();

            Ok(MintEffect::Success(nfts))
        }
        SuiExecutionStatus::Failure { error } => Ok(MintEffect::Error(error)),
    }
}

// TODO: Add back
// pub async fn handle_parallel_mint_nft(
//     wallet_ctx: Arc<WalletContext>,
//     package_id: Arc<String>,
//     module_name: Arc<String>,
//     gas_budget: Arc<u64>,
//     gas_coin: Arc<ObjectRef>,
//     sender: SuiAddress,
// ) -> JoinHandle<Result<Vec<ObjectID>, RustSdkError>> {
//     tokio::spawn(async move {
//         mint_nft_with_gas_coin(
//             wallet_ctx,
//             package_id,
//             module_name,
//             gas_budget,
//             gas_coin,
//             sender,
//         )
//         .await
//     })
// }

// TODO: Add back
// pub async fn mint_nft_with_gas_coin(
//     wallet_ctx: Arc<WalletContext>,
//     package_id: Arc<String>,
//     module_name: Arc<String>,
//     gas_budget: Arc<u64>,
//     gas_coin: Arc<ObjectRef>,
//     // SuiAddress implements Copy
//     sender: SuiAddress,
// ) -> Result<Vec<ObjectID>, RustSdkError> {
//     let package_id = ObjectID::from_str(package_id.as_str())
//         .map_err(|err| err::object_id(err, package_id.as_str()))?;

//     let mut retry = 0;
//     let response = loop {
//         let mut builder = ProgrammableTransactionBuilder::new();

//         // TODO: generalise amount of NFTs minted
//         for _ in 0..1_000 {
//             builder.move_call(
//                 package_id,
//                 Identifier::new(module_name.as_str()).unwrap(),
//                 Identifier::new("airdrop_nft").unwrap(),
//                 vec![],
//                 vec![],
//             )?;
//         }

//         let data = TransactionData::new_programmable(
//             sender,
//             vec![*gas_coin], // Gas Objects
//             builder.finish(),
//             *gas_budget, // Gas Budget
//             1_000,       // Gas Price
//         );

//         // Sign transaction.
//         let signatures: Vec<Signature> = vec![wallet_ctx
//             .config
//             .keystore
//             .sign_secure(&sender, &data, Intent::sui_transaction())?];

//         // Execute the transaction.
//         let response_ = wallet_ctx
//             .execute_transaction_block(
//                 Transaction::from_data(
//                     data,
//                     Intent::sui_transaction(),
//                     signatures,
//                 )
//                 .verify()?,
//             )
//             .await;

//         if retry == 3 {
//             break response_?;
//         }

//         if response_.is_err() {
//             println!("Retrying mint...");
//             let ten_millis = time::Duration::from_millis(1000);
//             thread::sleep(ten_millis);
//             retry += 1;
//             continue;
//         }
//         break response_?;
//     };

//     let SuiTransactionBlockEffects::V1(effects) = response.effects.unwrap();

//     let nft_ids = effects.created;
//     let mut i = 0;

//     let nfts = nft_ids
//         .iter()
//         .map(|obj_ref| {
//             i += 1;
//             obj_ref.reference.object_id
//         })
//         .collect::<Vec<ObjectID>>();

//     Ok(nfts)
// }
