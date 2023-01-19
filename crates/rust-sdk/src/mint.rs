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

pub const NFT_PROTOCOL: &str = "0xc3f1cfc87eae7fe79184005938f559953c804f4b";
// TODO: NEEDs to be updated
pub const SUI_MARINES: &str = "0xd797d3c7b587ec75e576426629d1d5e277bca088";
pub const GAS_OBJECT: &str = "0x871661956548f4ccc2e0531c8d46bc9f07d636f9";
pub const MY_ADDRESS: &str = "0xd8fb1b0ed0ddd5b3d07f3147d58fdc2eb880d143";

pub async fn create_inventory(
    sui: &SuiClient,
    keystore: &Keystore,
) -> Result<String, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(NFT_PROTOCOL)?;

    let call = sui
        .transaction_builder()
        .move_call::<sui_adapter::execution_mode::Normal>(
            address,
            package_id,
            "inventory",
            "init_inventory",
            vec![],
            vec![], // Should have tx context?
            None,   // Gas object, Node can pick on itself
            1000,   // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.

    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

    // We know `init_inventory` move function will create 1 object.
    let inventory_id = response
        .effects
        .unwrap()
        .created
        .first()
        .unwrap()
        .reference
        .object_id;

    println!("Created new inventory, with the id : [{}]", inventory_id);
    println!("Inventory owner: {}", address);

    Ok(inventory_id.to_string())
}

pub async fn get_client() -> Result<SuiClient, anyhow::Error> {
    let client = SuiClient::new("https://fullnode.devnet.sui.io:443", None, None).await?;

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
    inventory_id: ObjectID,
) -> Result<ObjectID, anyhow::Error> {
    let address = SuiAddress::from_str(MY_ADDRESS)?;
    let package_id = ObjectID::from_str(NFT_PROTOCOL)?;

    let call = sui
        .transaction_builder()
        .move_call::<sui_adapter::execution_mode::Normal>(
            address,
            package_id,
            "suimarines",
            "mint_nft",
            vec![],
            vec![
                SuiJsonValue::from_str(&name)?,
                SuiJsonValue::from_str(&description)?,
                SuiJsonValue::from_str(&url)?, // TODO: This should be Url
                SuiJsonValue::from_object_id(inventory_id),
            ], // Should have tx context?
            None, // Gas object, Node can pick on itself
            1000, // Gas budget
        )
        .await?;

    // Sign transaction.
    let signature = keystore.sign_secure(&address, &call, Intent::default())?;

    // Execute the transaction.

    let response = sui
        .quorum_driver()
        .execute_transaction(
            Transaction::from_data(call, Intent::default(), signature).verify()?,
            Some(ExecuteTransactionRequestType::WaitForLocalExecution),
        )
        .await?;

    assert!(response.confirmed_local_execution);

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
        "The NFT has been send to the following Inventory: [{}]",
        inventory_id.to_string()
    );
    println!("Inventory owner: {}", address);

    Ok(nft_id)
}
