use actix_web::{web, post, HttpResponse, Responder};
use gutenberg::generate_contract_with_path;
use tempfile::tempdir;
use walkdir::WalkDir;
use std::io::Write;

#[post("/generate-contract")]
pub async fn generate_contract(input_data: web::Bytes) -> impl Responder {
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
    let result = generate_contract_with_path(false, &input_file_path, &output_dir);

    if result.is_err() {
        return HttpResponse::InternalServerError().body("Failed to generate contract");
    }

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
    let move_file_data = match fs::read(&move_file_path) {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to read .move file"),
    };

    return HttpResponse::Ok()
        .header("Content-Disposition", format!("attachment; filename={}", move_file_path.file_name().unwrap().to_string_lossy()))
        .body(move_file_data);
}