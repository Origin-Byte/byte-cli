use std::{
    fs,
    io::{Read, Write},
};

use actix_web::{post, web, HttpResponse, Responder, Result};
use package_manager::Network;
use rust_sdk::publish;
use serde::Deserialize;
use sui_sdk::{
    rpc_types::Coin,
    types::{base_types::SuiAddress, transaction::TransactionData},
};

use crate::err::ByteApiError;
use crate::io;

#[derive(Deserialize)]
pub struct RequestData {
    name: String,
    project_dir: String,
    config_json: String,
    sender: SuiAddress,
    gas_budget: u64,
    gas_coin: Coin,
    network: Network,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/gen-build-publish-tx")]
pub async fn gen_build_publish_tx(
    data: web::Json<RequestData>,
) -> Result<impl Responder> {
    // Input
    let contract_dir =
        io::get_contract_path(&data.name, &Some(data.project_dir.to_owned()));

    let mut schema = serde_json::from_str(&data.config_json)?;

    gutenberg::generate_project_with_flavors(
        false,
        &mut schema,
        &contract_dir,
        Some(String::from("1.3.0")),
    )
    .map_err(ByteApiError::from)?;

    match data.network {
        Network::Mainnet => {}
        Network::Testnet => {
            let flavour_path = contract_dir.join("flavours/Move-test.toml");
            // Open the source file for reading
            let mut source_file = fs::File::open(flavour_path)?;

            // Create or open the destination file for writing
            let mut destination_file =
                fs::File::create(contract_dir.join("Move.toml"))?;

            // Read the contents of the source file
            let mut buffer = Vec::new();
            source_file.read_to_end(&mut buffer)?;

            // Output
            // Write the contents to the destination file
            destination_file.write_all(&buffer)?;
        }
    }

    let gas_coin = &data.gas_coin;

    let tx_data = publish::prepare_publish_contract(
        data.sender,
        contract_dir.as_path(),
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest),
        data.gas_budget,
    )
    .await
    .map_err(ByteApiError::from)?;

    let tx_data = match tx_data {
        TransactionData::V1(tx_data) => tx_data,
    };

    Ok(HttpResponse::Ok().json(tx_data))
}
