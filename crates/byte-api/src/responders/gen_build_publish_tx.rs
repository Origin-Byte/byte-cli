use actix_web::{post, web, HttpResponse, Responder};
use rust_sdk::publish;
use sui_sdk::types::{base_types::SuiAddress, transaction::TransactionData};
use serde::Deserialize;

use crate::io;

#[derive(Deserialize)]
pub struct RequestData {
    name: String,
    project_dir: String,
    config_json: String,
    sender: SuiAddress,
    gas_budget: u64,
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
) -> impl Responder {
    // Input
    let contract_dir =
        io::get_contract_path(&data.name, &Some(data.project_dir.to_owned()));

    let schema_res = serde_json::from_str(&data.config_json);

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
            .body(format!("Failed to generate contract: {:?}", result.err().unwrap()));
    }

    let tx_data_res = publish::prepare_publish_contract(
        data.sender,
        contract_dir.as_path(),
        None,
        data.gas_budget,
    )
    .await;

    if tx_data_res.is_err() {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to prepare contract publishing transaction: {:?}", result.err().unwrap()));
    }

    let tx_data = tx_data_res.unwrap();

    let tx_data = match tx_data {
        TransactionData::V1(tx_data) => tx_data,
    };

    return HttpResponse::Ok().json(tx_data);
}
