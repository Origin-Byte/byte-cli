use actix_web::{post, web, HttpResponse, Responder, Result};
use anyhow::anyhow;
use serde::Deserialize;
use std::fs;
use walkdir::WalkDir;

use crate::{err::ByteApiError, io};

#[derive(Deserialize)]
pub struct RequestData {
    name: String,
    project_dir: String,
    config_json: String,
}

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/gen-contract")]
pub async fn gen_contract(
    data: web::Json<RequestData>,
) -> Result<impl Responder> {
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

    // Search for the .move file in the output directory
    let move_file_path = WalkDir::new(&data.project_dir)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_string_lossy().ends_with(".move"))
        .map(|entry| entry.into_path());

    if move_file_path.is_none() {
        return Err(ByteApiError::AnyhowError(anyhow!(
            "Failed to generate .move file"
        ))
        .into());
    }

    // Safe to unwrap
    let move_file_path = move_file_path.unwrap();

    // Read the .move file and return it in the response
    let move_file_data = fs::read(&move_file_path)?;

    return Ok(HttpResponse::Ok()
        .append_header((
            "Content-Disposition",
            format!(
                "attachment; filename={}",
                move_file_path.file_name().unwrap().to_string_lossy()
            ),
        ))
        .body(move_file_data));
}
