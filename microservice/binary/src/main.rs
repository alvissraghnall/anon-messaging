use actix_web::{web, App, HttpResponse, HttpServer, Responder};
//use api::key_generation::generate_keys;
use db::db::create_db_pool;
use serde::{Deserialize, Serialize};
use std::{env, sync::Arc};
use api::{user::{configure_routes as configure_user_routes, UserController, UserControllerImpl}};
use service::{user::UserService};
use api::token::TokenControllerImpl;
use service::message::{
    repository::MessageRepository,
    service::MessageService,
};
use api::message::{
    configure_routes as configure_message_routes,
    MessageController,
    MessageControllerImpl
};
use utoipauto::utoipauto;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[utoipauto(
    paths = "./api/src/ from api",
    schemas = "./shared/src/ from shared"
)]
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "anon", description = "Anon messaging endpoints.")
    )
)]
pub struct ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let pool = create_db_pool()
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    sqlx::migrate!("../db/migrations")
        .run(&pool)
        .await;

    let user_service = UserService::new(pool.clone());
    let user_controller: Arc<dyn UserController> = Arc::new(UserControllerImpl::new(web::Data::new(user_service)));

    let message_service = web::Data::new(MessageService::new(pool.clone()));
    let message_controller = Arc::new(
        MessageControllerImpl::new(message_service)
    ) as Arc<dyn MessageController>;
    
    HttpServer::new(move || {
        let user_controller = user_controller.clone();
        let message_controller = message_controller.clone();
        let token_repo = pool.clone();
        
        App::new()
            .app_data(web::Data::new(user_controller))
            .app_data(web::Data::new(message_controller))
            .configure(configure_message_routes)
            .configure(configure_user_routes)
            .configure(TokenControllerImpl::configure(token_repo))
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", ApiDoc::openapi()),
            )
            .route("/healthz", web::get().to(health_check))
//            .route("/generate-keys", web::post().to(generate_keys))
    })
    .bind(("0.0.0.0", 10000))?
    .run()
    .await
}

async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("OK")
}
