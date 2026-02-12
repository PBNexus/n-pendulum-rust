// src/main.rs
use actix_files::Files;
use actix_web::{middleware, web, App, HttpServer};
use std::env;

mod logic;
mod math;
mod ui;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Initialize the logger so Actix can output to the console
    // "info" means show all info, warnings, and errors.
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    println!("Starting server on 0.0.0.0:{}", port);

    HttpServer::new(|| {
        App::new()
            // 2. Wrap the app in the Logger middleware
            .wrap(middleware::Logger::default())
            .route("/simulate", web::post().to(ui::simulate_handler))
            .service(
                Files::new("/", "./static")
                    .index_file("index.html")
                    .use_last_modified(true),
            )
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}