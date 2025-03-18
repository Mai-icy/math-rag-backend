use actix_web::{web, HttpServer, App};
use actix_web::middleware::from_fn;
use database::init_pool;
use crate::handlers::{handle_login, handle_register, index};
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
            .route("/login", web::post().to(handle_login)),
    );
    cfg.service(
        web::scope("/v1/chat")
            .wrap(from_fn(auth_middleware))
            .route("/index", web::get().to(index))
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

