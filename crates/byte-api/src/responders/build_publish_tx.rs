use actix_web::{post, web, HttpResponse, Responder};
use dotenv::dotenv;
use rust_sdk::{coin::select_biggest_coin, publish, utils::get_context};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::Path};
use sui_sdk::types::{base_types::SuiAddress, transaction::TransactionData};

#[derive(Deserialize)]
pub struct RequestData {
    sender: SuiAddress,
    gas_budget: u64,
    contract_dir: String,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/build-publish-tx")]
pub async fn build_publish_tx(data: web::Json<RequestData>) -> impl Responder {
    dotenv().ok();

    let contract_dir = Path::new(&data.contract_dir);

    let wallet_ctx = get_context().await.unwrap();
    let client = wallet_ctx.get_client().await.unwrap();
    let gas_coin = select_biggest_coin(&client, data.sender).await.unwrap();

    let tx_data_res = publish::prepare_publish_contract(
        data.sender,
        contract_dir,
        (gas_coin.coin_object_id, gas_coin.version, gas_coin.digest),
        data.gas_budget,
    )
    .await;

    if tx_data_res.is_err() {
        return HttpResponse::InternalServerError()
            .body("Failed to prepare contract publishing transaction");
    }

    let tx_data = tx_data_res.unwrap();

    let tx_data = match tx_data {
        TransactionData::V1(tx_data) => tx_data,
    };

    return HttpResponse::Ok().json(tx_data);
}

#[derive(Serialize)]
struct TransactionDataWrapper {
    inner: TransactionData,
}
