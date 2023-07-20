use actix_web::{post, web, HttpResponse, Responder, Result};
use dotenv::dotenv;
use rust_sdk::publish;
use serde::{Deserialize, Serialize};
use std::path::Path;
use sui_sdk::{
    rpc_types::Coin,
    types::{base_types::SuiAddress, transaction::TransactionData},
};

use crate::err::ByteApiError;

#[derive(Deserialize)]
pub struct RequestData {
    sender: SuiAddress,
    gas_budget: u64,
    contract_dir: String,
    gas_coin: Coin,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/build-publish-tx")]
pub async fn build_publish_tx(
    data: web::Json<RequestData>,
) -> Result<impl Responder> {
    dotenv().ok();

    let contract_dir = Path::new(&data.contract_dir);

    let gas_coin = &data.gas_coin;

    let tx_data = publish::prepare_publish_contract(
        data.sender,
        contract_dir,
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

#[derive(Serialize)]
struct TransactionDataWrapper {
    inner: TransactionData,
}
