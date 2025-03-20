use std::str::FromStr;

use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest};
use actix_web::{web, HttpResponse, Responder, http::header};
use crate::models::*;
use crate::database::*;
use crate::utils::{generate_jwt, generate_uuid};
use uuid::Uuid;
use serde_json::{json, Value};
use bcrypt::verify;

pub async fn index(req: HttpRequest) -> impl Responder {
    // for test
    let is_auth = req.extensions().get::<bool>().unwrap().clone();
    if !is_auth{
        return HttpResponse::NotFound().body("not good");
    }

    let session = req.extensions().get::<Session>().unwrap().clone();

    print!("{}", session.session_id.to_string());
    let mut text = String::from("hello");
    text.push_str(&session.session_id.to_string());
    HttpResponse::Ok().body(text)
}

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
    let user = get_user_by_username(&pool, &payload.username);

    if !matches!(user, Err(diesel::result::Error::NotFound)) {
        return HttpResponse::Conflict().json(json!({"message": "用户名已被注册"}));
    }

    let new_user = NewUser::new(&payload.username, &payload.email, &payload.password);

    if let Err(err) = add_new_user(&pool, &new_user) {
        HttpResponse::InternalServerError().json(json!({"message": err.to_string()}))
    }else{
        HttpResponse::Ok().json(json!({"message": "注册成功"}))
    }  
}

pub async fn chat_new(
    req: HttpRequest,
    pool: web::Data<DbPool>,
    payload: web::Json<NewChatPayload>, 
) -> impl Responder {
    let is_auth = req.extensions().get::<bool>().unwrap().clone();
    if !is_auth{
        return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"}));
    }

    if payload.title.len() < 3 {
        return HttpResponse::BadRequest().json(json!({"message": "标题过短"}));
    }

    let session = req.extensions().get::<Session>().unwrap().clone();
    let new_chat = NewChat::new(session.user_id, &payload.title);

    if let Err(err) = add_new_chat(&pool, &new_chat){
        return HttpResponse::InternalServerError().json(json!({"message": err.to_string()}))
    }else{
        HttpResponse::Ok().json(json!({"message": "创建对话成功"}))
    }
}

pub async fn chat_history(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let is_auth = req.extensions().get::<bool>().unwrap().clone();
    if !is_auth{
        return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"}));
    }
    let session = req.extensions().get::<Session>().unwrap().clone();

    let chats = match get_all_chats_by_user_id(&pool, session.user_id){
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().json(json!({"message": err.to_string()}))
    };

    let mut chats_fliter: Vec<Value> = vec![];

    for chat in chats {
        let chat_json = json!({
            "chat_id": chat.chat_id.to_string(),
            "title": chat.title,
            "created_at": chat.created_at.unwrap().to_string()
        });
        chats_fliter.push(chat_json);
    }

    let response = json!({
        "chats": chats_fliter,
        "status": "200",
        "message": "查询历史对话成功"
    });

    HttpResponse::Ok().json(response)
}

//  /v1/chat/{chat_id}
pub async fn chat_content(
    chat_id: web::Path<String>,
    pool: Data<DbPool>,
) -> impl Responder{

    let chat_uuid = match Uuid::from_str(&chat_id.into_inner()) {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().json(json!({"message": "uuid 不合法"}))
    };

    let msgs = match get_all_messages_by_chat_id(&pool, chat_uuid) {
        Ok(data) => data,
        Err(err) => return HttpResponse::InternalServerError().json(json!({"message": err.to_string()}))
    };

    let mut msgs_json: Vec<Value> = vec![];
    
    for msg in msgs{
        let msg_json = json!({
            "role": msg.role,
            "content": msg.content,
            "timestamp": msg.timestamp.unwrap().to_string()
        });
        msgs_json.push(msg_json);
    }

    let response = json!({
        "chats": msgs_json,
        "status": "200",
        "message": "查询对话内容成功"
    });

    HttpResponse::Ok().json(json!(response))
}

pub async fn chat_delete(
    chat_id: web::Path<String>,
    pool: Data<DbPool>,
) -> impl Responder {
    let chat_uuid = match Uuid::from_str(&chat_id.into_inner()) {
        Ok(data) => data,
        Err(_) => return HttpResponse::BadRequest().json(json!({"message": "uuid 不合法"})),
    };

    match delete_chat(&pool, chat_uuid) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "对话删除成功"})),
        Err(_) => HttpResponse::InternalServerError().json(json!({"message": "删除失败"})),
    }
}
