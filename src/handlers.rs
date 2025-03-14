use actix_web::{web, HttpResponse, Responder, http::header};
use crate::models::*;
use crate::database::*;
use crate::utils::generate_jwt;
use crate::utils::{generate_uuid};
use serde_json::json;
use bcrypt::verify;

pub async fn handle_login(
    pool: web::Data<DbPool>,
    payload: web::Json<LoginPayload>,
) -> impl Responder {


    let user = match get_user_by_username(&pool, &payload.username) {
        Ok(user) => user,
        Err(diesel::result::Error::NotFound) => return HttpResponse::NotFound().json(json!({"message": "用户不存在"})),
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };
    
    
    if verify(&payload.password, &user.password_hash).unwrap() {
        let session_id = generate_uuid();

        let jwt_token = match generate_jwt(&session_id) {
            Ok(jwt_token) => jwt_token,
            Err(err) => return HttpResponse::InternalServerError().json(err.to_string()), 
        };

        let new_session = NewSession::new(session_id, user.user_id, &jwt_token);
        let _ = add_new_session(&pool, &new_session);

        HttpResponse::Ok()
            .insert_header((header::AUTHORIZATION, format!("Bearer {}", jwt_token)))
            .json("Login successful")
    }else{
        HttpResponse::Unauthorized().json(json!({"message": "密码不正确"}))
    }

}

pub async fn handle_register(
    pool: web::Data<DbPool>,
    payload: web::Json<RegisterPayload>,
) -> impl Responder {
    println!("received {} {}", payload.username, payload.password);

    let user = get_user_by_username(&pool, &payload.username);

    if !matches!(user, Err(diesel::result::Error::NotFound)) {
        return HttpResponse::Conflict().json(json!({"message": "用户名已被注册"}));
    }

    let new_user = NewUser::new(&payload.username, &payload.email, &payload.password);

    let _ = add_new_user(&pool, &new_user);

    HttpResponse::Ok().json(json!({"message": "注册成功"}))
}
