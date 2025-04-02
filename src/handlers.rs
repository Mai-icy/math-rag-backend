use std::str::FromStr;

use reqwest::Client;
use actix_web::web::Data;
use actix_web::{HttpMessage, HttpRequest};
use actix_web::{web, HttpResponse, Responder, http::header};
use crate::models::*;
use crate::database::*;
use crate::utils::{generate_jwt, generate_uuid};
use uuid::Uuid;
use serde_json::{json, Value};
use bcrypt::verify;
use futures::TryStreamExt;
use tokio::sync::mpsc;
use bytes::Bytes;
use crate::utils::{decode_jwt, now};
use crate::xunfei_ocr::img2latex;

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


pub async fn handle_logout(
    req: HttpRequest,
    pool: web::Data<DbPool>,
) -> impl Responder {
    let auth_header = req.headers().get("Authorization");

    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(t) => t.to_string(),
        None => return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"})),
    };

    let session_uuid = match decode_jwt(&token) {
        Ok(uuid) => uuid,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"})),
    };

    let _ = match get_session_by_session_id(&pool, session_uuid) {
        Ok(session) => session,
        Err(_) => return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"})),
    };

    let response = match update_expries_by_session(&pool, session_uuid, now()) {
        Ok(_) => HttpResponse::Ok().json(json!({"message": "已成功退出登录"})),
        Err(_) => HttpResponse::Unauthorized().json(json!({"message": "登出失败"}))
    };
    response
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

pub async fn proxy_stream(
    req_body: web::Json<ChatPayload>,
    pool: Data<DbPool>
) -> impl Responder {
    let client = Client::new();
    let url = "http://localhost:8000/stream";

    let prompt = req_body.prompt.clone();
    let chat_id = Uuid::from_str(&req_body.chat_id).unwrap();

    let user_msg: NewMessage = NewMessage::new(chat_id, &String::from("user"), &prompt);
    
    let _ = add_new_message(&pool, &user_msg);


    let res = match client.post(url).json(&req_body).send().await {
        Ok(res) => res,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to connect to backend"),
    };

    let (tx, mut rx) = mpsc::channel::<Bytes>(10);
    let mut collected_response = Vec::new();

    let mut stream = res.bytes_stream();

    tokio::spawn(async move {
        while let Some(chunk) = stream.try_next().await.unwrap_or(None) {
            collected_response.extend_from_slice(&chunk);

            if tx.send(Bytes::from(chunk)).await.is_err() {
                break;
            }
        }

        // println!("{}", String::from_utf8_lossy(&collected_response));
        let json_str = String::from_utf8_lossy(&collected_response).into_owned();
        let parsed: Value = serde_json::from_str(&json_str).unwrap();

        let result: String = parsed["response"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["chunk"].as_str().unwrap_or(""))
            .collect();

        let ai_msg: NewMessage = NewMessage::new(chat_id, &String::from("assistant"), &result);
        let _ = add_new_message(&pool, &ai_msg);

        println!("{}", result);
    });

    let response_stream = async_stream::stream! {
        while let Some(chunk) = rx.recv().await {
            yield Ok::<Bytes, actix_web::Error>(chunk);
        }
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .streaming(response_stream)
}

pub async fn ocr_handle(
    req: HttpRequest,
    payload: web::Json<OCRPalyload>,
) -> impl Responder{
    let is_auth = req.extensions().get::<bool>().unwrap().clone();
    if !is_auth{
        return HttpResponse::Unauthorized().json(json!({"message": "用户未登录"}));
    }

    let result: String = match img2latex(&payload.imgb64).await{
        Ok(data) => data,
        Err(err) => return HttpResponse::BadRequest().json(json!({"message": err.to_string()})),
    };

    HttpResponse::BadRequest().json(
        json!({"message": "扫描成功", "content": result}))
    

}
