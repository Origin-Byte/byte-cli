use actix_web::{post, web, HttpResponse, Responder};
use dotenv::dotenv;
use rust_sdk::publish;
use serde::Serialize;
use std::{ffi::OsString, path::Path};
use sui_sdk::types::{base_types::SuiAddress, transaction::TransactionData};

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/build-publish-tx")]
pub async fn build_publish_tx(
    sender: web::Bytes,
    gas_budget: web::Bytes,
    contract_dir: web::Path<String>,
) -> impl Responder {
    dotenv().ok();
    let sender = match SuiAddress::from_bytes(sender.as_ref()) {
        Ok(sender) => sender,
        Err(_) => {
            return HttpResponse::BadRequest().body("Invalid 'sender' address");
        }
    };

    let gas_budget = match std::str::from_utf8(gas_budget.as_ref()) {
        Ok(gas_budget) => gas_budget,
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Invalid UTF-8 argument: 'gas_budget'");
        }
    };

    let gas_budget = match gas_budget.parse::<u64>() {
        Ok(gas_budget) => gas_budget,
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Unable to pares gas_budget into u64 ");
        }
    };

    let contract_dir = OsString::from(contract_dir.as_ref());
    // Convert the `OsString` into a `Path`
    let contract_dir = Path::new(&contract_dir);

    let tx_data_res = publish::prepare_publish_contract(
        sender,
        contract_dir,
        None,
        gas_budget,
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
