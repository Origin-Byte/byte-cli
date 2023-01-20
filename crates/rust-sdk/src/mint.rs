use std::str::FromStr;
use sui_keys::keystore::{AccountKeystore, FileBasedKeystore, Keystore};
use sui_sdk::{
    json::SuiJsonValue,
    types::{
        base_types::{ObjectID, SuiAddress},
        messages::Transaction,
    },
    SuiClient,
};
use sui_types::intent::Intent;
use sui_types::messages::ExecuteTransactionRequestType;

pub const NFT_PROTOCOL: &str = "0x31cea8513aa6d078d92823ea42d42600208be721";
pub const SUI_MARINES: &str = "0x8c4d633c3834265522d3410571da19f3b0920054";

pub const GAS_OBJECT: &str = "0x0c22130a404e235a29c6892d1dfb3915527f3208";
pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

pub async fn create_warehouse(
    sui: &SuiClient,
    keystore: &Keystore,
) -> Result<String, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(NFT_PROTOCOL)?;

    let call = sui
        .transaction_builder()
        .move_call(
            address,
            package_id,
            "warehouse",
            "init_warehouse",
            vec![],
            vec![], // Should have tx context?
            None,   // Gas object, Node can pick on itself
            10000,  // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    let data_chato = Transaction::from_data(
        call.clone(),
        Intent::default(),
        signature.clone(),
    )
    .verify()?;

    // Execute the transaction.
    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature)
                .verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

    // We know `init_warehouse` move function will create 1 object.
    let warehouse_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new warehouse, with the id : [{}]", warehouse_id);
    println!("Warehouse owner: {}", address);

    Ok(warehouse_id.to_string())
}

pub async fn get_client() -> Result<SuiClient, anyhow::Error> {
    let client =
        SuiClient::new("https://fullnode.devnet.sui.io:443", None, None)
            .await?;

    Ok(client)
}

pub async fn get_keystore() -> Result<Keystore, anyhow::Error> {
    // Load keystore from ~/.sui/sui_config/sui.keystore
    let keystore_path = match dirs::home_dir() {
        Some(v) => v.join(".sui").join("sui_config").join("sui.keystore"),
        None => panic!("Cannot obtain home directory path"),
    };

    let keystore = Keystore::from(FileBasedKeystore::new(&keystore_path)?);

    Ok(keystore)
}

pub async fn mint_nft(
    sui: &SuiClient,
    keystore: &Keystore,
    name: &str,
    description: &str,
    url: &str,
    warehouse_id: &str,
    package_id: &str,
) -> Result<ObjectID, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(package_id)?;
    let warehouse_id = ObjectID::from_str(warehouse_id)?;

    println!("Package ID: {}", package_id);

    let call = sui
        .transaction_builder()
        .move_call(
            address,
            package_id,
            "suimarines",
            "mint_nft",
            vec![],
            vec![
                SuiJsonValue::from_str(&name)?,
                SuiJsonValue::from_str(&description)?,
                SuiJsonValue::from_str(&url)?, // TODO: This should be Url
                SuiJsonValue::from_object_id(warehouse_id),
            ], // Should have tx context?
            None,   // Gas object, Node can pick on itself
            100000, // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.

    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature)
                .verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

    println!("{:?}", response);

    // We know `mint_nft` move function will create 1 object.
    let nft_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new NFT, with the id : [{}]", nft_id);
    println!(
        "The NFT has been send to the following Warehouse: [{}]",
        warehouse_id.to_string()
    );
    println!("Warehouse owner: {}", address);

    Ok(nft_id)
}

// pub async fn collect_royalties() {}
