use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;
use walkdir::WalkDir;
use std::fs;
use gutenberg::{Schema, generate_contract_with_path};
use crate::fs::File;
use std::ffi::OsStr;

// Define a handler for the root path ("/")
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("To replace with swagger docs")
}

#[post("/generate-contract")]
async fn generate_contract_endpoint(input_data: web::Bytes) -> impl Responder {
    // Create temporary directories
    let temp_dir = tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&input_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    // Write the JSON data to a file in the input directory
    let input_file_path = input_dir.join("input.json");
    let mut f = fs::File::create(&input_file_path).unwrap();
    f.write_all(&input_data).unwrap();

    // Call generate_contract
    generate_contract_with_path(false, &input_file_path, &output_dir);

    // Search for the .move file in the output directory
    let move_file_path = WalkDir::new(&output_dir)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_string_lossy().ends_with(".move"))
        .map(|entry| entry.into_path());

    // Check if the .move file was found
    let move_file_path = match move_file_path {
        Some(path) => path,
        None => return HttpResponse::InternalServerError().body("Failed to generate .move file"),
    };

    // Read the .move file and return it in the response
    let move_file_data = fs::read(move_file_path.clone()).unwrap();
    return HttpResponse::Ok()
        .header("Content-Disposition", format!("attachment; filename={}", move_file_path.file_name().unwrap().to_string_lossy()))
        .body(move_file_data);
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
    .unwrap_or_else(|_| "8080".to_string())
    .parse::<u16>()
    .unwrap();
    println!("Starting server at http://0.0.0.0:{}", port);

    // Start the HTTP server
    HttpServer::new(|| {
        App::new()
        .service(index)
        .service(generate_contract_endpoint)
    })
    .bind(("0.0.0.0", port))? // Bind to the desired host and port
    .run()
    .await
}
