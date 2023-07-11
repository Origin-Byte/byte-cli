mod io;
mod responders;

use crate::responders::gen_build_publish_tx::gen_build_publish_tx;
use actix_web::{get, App, HttpResponse, HttpServer, Responder};

// Define a handler for the root path ("/")
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("To replace with swagger docs")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();
    println!("Starting server at http://0.0.0.0:{}", port);

    // Start the HTTP server
    HttpServer::new(|| App::new().service(index).service(gen_build_publish_tx))
        .bind(("0.0.0.0", port))? // Bind to the desired host and port
        .run()
        .await
}
