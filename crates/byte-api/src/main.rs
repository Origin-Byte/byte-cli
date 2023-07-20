mod err;
mod io;
mod responders;

use actix_web::{middleware::Logger, App, HttpServer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::responders::gen_build_publish_tx::gen_build_publish_tx;

#[derive(OpenApi)]
#[openapi(
        paths(
            responders::gen_contract::gen_contract,
            responders::build_publish_tx::build_publish_tx,
            responders::gen_build_publish_tx::gen_build_publish_tx,
        ),
        tags(
            (name = "gen_contract", description = "Generate contracts"),
            (name = "build_publish_tx", description = "Build Publish Transactions"),
            (name = "gen_build_publish_tx", description = "Generated contracts and build Publish Transactions")
        )
    )]
struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    // Make instance variable of ApiDoc so all worker threads gets the same instance.
    let openapi = ApiDoc::openapi();

    let factory = move || {
        // This factory closure is called on each worker thread independently.
        App::new()
            .wrap(Logger::default())
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", openapi.clone()),
            )
            .service(gen_build_publish_tx)
    };

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap();

    println!("Starting server at http://0.0.0.0:{}/swagger-ui/", port);

    // Start the HTTP server
    HttpServer::new(factory)
        .bind(("0.0.0.0", port))? // Bind to the desired host and port
        .run()
        .await
}
