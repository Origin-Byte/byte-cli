use actix_web::{post, web, HttpResponse, Responder};
use anyhow::Result;
use std::fs;
use walkdir::WalkDir;

use crate::io;

#[utoipa::path(
    responses(
        (status = 201, description = "Success!"),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request")
    ),
)]
#[post("/gen-contract")]
pub async fn gen_contract(
    name: web::Bytes,
    project_dir: web::Bytes,
    config_json: web::Bytes,
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

    // Search for the .move file in the output directory
    let move_file_path = WalkDir::new(project_dir)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_string_lossy().ends_with(".move"))
        .map(|entry| entry.into_path());

    // Check if the .move file was found
    let move_file_path = match move_file_path {
        Some(path) => path,
        None => {
            return HttpResponse::InternalServerError()
                .body("Failed to generate .move file")
        }
    };

    // Read the .move file and return it in the response
    let move_file_data = match fs::read(&move_file_path) {
        Ok(data) => data,
        Err(_) => {
            return HttpResponse::InternalServerError()
                .body("Failed to read .move file")
        }
    };

    return HttpResponse::Ok()
        .append_header((
            "Content-Disposition",
            format!(
                "attachment; filename={}",
                move_file_path.file_name().unwrap().to_string_lossy()
            ),
        ))
        .body(move_file_data);
}
