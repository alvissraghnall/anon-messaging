use api::key_generation::generate_keys;
use db::db::{create_db_pool};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

	env::set_var("RUST_LOG", "debug");
    env_logger::init();

	let pool = create_db_pool().await.map_err(|e| std::io::Error::new(
    	std::io::ErrorKind::Other,
	    e.to_string()
	))?;

	HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/generate-keys", web::post().to(generate_keys))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
