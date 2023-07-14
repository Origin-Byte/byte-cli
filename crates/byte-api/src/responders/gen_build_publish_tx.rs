use std::str::FromStr;

use actix_web::{post, web, HttpResponse, Responder};
use rust_sdk::publish;
use sui_sdk::types::{base_types::SuiAddress, transaction::TransactionData};

use crate::io;

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/gen-build-publish-tx")]
pub async fn gen_build_publish_tx(
    name: web::Bytes,
    project_dir: web::Bytes,
    config_json: web::Bytes,
    sender: web::Bytes,
    gas_budget: web::Bytes,
) -> impl Responder {
    // Convert Bytes into &str
    let name = match std::str::from_utf8(name.as_ref()) {
        Ok(name) => name,
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Invalid UTF-8 argument: 'name'");
        }
    };

    let project_dir = match std::str::from_utf8(project_dir.as_ref()) {
        Ok(project_dir) => project_dir,
        Err(_) => {
            return HttpResponse::BadRequest()
                .body("Invalid UTF-8 argument: 'project_name'");
        }
    };

    let sender_str = match std::str::from_utf8(sender.as_ref()) {
        Ok(sender) => sender,
        Err(_) => {
            return HttpResponse::BadRequest().body(format!("Invalid 'sender': {:?}", sender));
        }
    };

    let sender = match SuiAddress::from_str(sender_str) {
        Ok(sender) => sender,
        Err(_) => {
            return HttpResponse::BadRequest().body(format!("Invalid 'sender' str address: {:?}", sender_str));
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

    // Input
    let contract_dir =
        io::get_contract_path(name, &Some(String::from(project_dir)));

    let schema_res = serde_json::from_slice(&config_json);

    if schema_res.is_err() {
        return HttpResponse::InternalServerError()
            .body("Failed to parse config json");
    }

    let mut schema = schema_res.unwrap();

    let result = gutenberg::generate_project_with_flavors(
        false,
        &mut schema,
        &contract_dir,
        Some(String::from("1.3.0")),
    );

    if result.is_err() {
        return HttpResponse::InternalServerError()
            .body("Failed to generate contract");
    }

    let tx_data_res = publish::prepare_publish_contract(
        sender,
        contract_dir.as_path(),
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
