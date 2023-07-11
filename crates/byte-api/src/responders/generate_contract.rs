use actix_web::{post, web, HttpResponse, Responder};
use anyhow::{anyhow, Result};
use gutenberg_types::Schema;
use std::{
    fs::{self, File},
    path::Path,
};
use walkdir::WalkDir;

use crate::io;

#[post("/generate-contract")]
pub async fn generate_contract(
    name: web::Bytes,
    project_dir: web::Bytes,
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
    let schema_path =
        io::get_schema_filepath(name, &Some(String::from(project_dir)));
    let contract_dir =
        io::get_contract_path(name, &Some(String::from(project_dir)));

    // Logic
    let schema_res = parse_config(schema_path.as_path());

    if schema_res.is_err() {
        return HttpResponse::InternalServerError()
            .body("Failed to parse contract schema");
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

pub fn parse_config(config_file: &Path) -> Result<Schema> {
    let file = File::open(config_file).map_err(|err| {
        anyhow!(
            r#"Could not find configuration file "{}": {err}
Call `byte-cli init-collection-config` to initialize the configuration file."#,
            config_file.display()
        )
    })?;

    serde_json::from_reader::<File, Schema>(file).map_err(|err|anyhow!(r#"Could not parse configuration file "{}": {err}
Call `byte-cli init-collection-config to initialize the configuration file again."#, config_file.display()))
}
