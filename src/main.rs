use actix_web::web::route;
use actix_web::{web, HttpServer, App};
use actix_web::middleware::from_fn;
use database::init_pool;
use handlers::*;
use middleware::auth_middleware;

mod database;
mod schema;
mod handlers;
mod models;
mod utils;
mod middleware;
mod xunfei_ocr;


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(handle_register))
            .route("/login", web::post().to(handle_login))
            .route("/logout", web::post().to(handle_logout))
    );
    cfg.service(
        web::scope("/v1/chat")
            .wrap(from_fn(auth_middleware))
            .route("/index", web::get().to(index))
            .route("/new", web::get().to(chat_new))
            .route("/history", web::get().to(chat_history))
            .route("/{chat_id}", web::get().to(chat_content))
            .route("/{chat_id}", web::delete().to(chat_delete))
    );
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool_data = init_pool();
    println!("Server started at http://127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool_data.clone()))
            .configure(config)
            .route("/", web::get().to(index).wrap(from_fn(auth_middleware)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

