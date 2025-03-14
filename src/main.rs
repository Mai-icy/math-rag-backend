use actix_web::{web, HttpServer, App};
use database::{add_new_user, init_pool};
use models::NewUser;
use schema::users::password_hash;
use crate::handlers::{handle_login, handle_register};

mod database;
mod schema;
mod handlers;
mod models;
mod utils;


pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/v1/auth")
            .route("/register", web::post().to(handle_register))
            .route("/login", web::post().to(handle_login)),
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

