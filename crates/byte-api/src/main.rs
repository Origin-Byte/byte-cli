use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};

// Define a handler for the root path ("/")
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().body("To replace with swagger docs")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Start the HTTP server
    HttpServer::new(|| {
        App::new().service(index) // Register the handler for the root path
    })
    .bind("127.0.0.1:8000")? // Bind to the desired host and port
    .run()
    .await
}
