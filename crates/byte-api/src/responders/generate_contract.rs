use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_multipart::Multipart;
use futures::StreamExt;
use std::io::Write;
use std::fs::read_dir;

async fn generate_contract_endpoint(mut payload: Multipart) -> impl Responder {
    // Create temporary directories
    let temp_dir = tempfile::tempdir().unwrap();
    let input_dir = temp_dir.path().join("input");
    let output_dir = temp_dir.path().join("output");
    std::fs::create_dir_all(&input_dir).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();

    // Handle the uploaded file
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition().unwrap();
        let filename = content_disposition.get_filename().unwrap();
        let filepath = input_dir.join(filename);
        let mut f = web::block(|| std::fs::File::create(filepath)).await.unwrap();

        // Write the uploaded file data to a file in the input directory
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            f = web::block(move || f.write_all(&data).map(|_| f)).await.unwrap();
        }

        // Call generate_contract
        generate_contract(&input_dir, &output_dir);

        // Read the output file(s) and return them in the response
        for entry in read_dir(output_dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let file_data = std::fs::read(path).unwrap();
                return HttpResponse::Ok().body(file_data);
            }
        }
    }

    HttpResponse::InternalServerError().finish()
}

fn generate_contract(config_path: &Path, output_dir: &Path) {
    let schema = assert_schema(config_path);

    // Create main contract directory
    let package_name = schema.package_name();
    let contract_dir = output_dir.join(&package_name);
    let sources_dir = contract_dir.join("sources");

    // Create directories
    std::fs::create_dir_all(&sources_dir).unwrap();

    // Create `Move.toml`
    let move_file = File::create(contract_dir.join("Move.toml")).unwrap();
    schema
        .write_move_toml(move_file)
        .expect("Could not write `Move.toml`");

    let module_name = schema.nft().module_name();
    let module_file =
        File::create(sources_dir.join(format!("{module_name}.move"))).unwrap();
    schema
        .write_move(module_file)
        .expect("Could not write Move module");
}