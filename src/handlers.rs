use actix_web::{HttpMessage, HttpRequest};
use actix_web::{web, HttpResponse, Responder, http::header};
use crate::models::*;
use crate::database::*;
use crate::schema::chats::title;
use crate::utils::{generate_jwt, generate_uuid};
use uuid::Uuid;
use serde_json::json;
use bcrypt::verify;

pub async fn index(req: HttpRequest) -> impl Responder {
    // for test
    let is_auth = req.extensions().get::<bool>().unwrap().clone();
    if !is_auth{
        return HttpResponse::NotFound().body("not good");
    }

    let sessionid = req.extensions().get::<Uuid>().unwrap().clone();
    print!("{}", sessionid.to_string());
    let mut text = String::from("hello");
    text.push_str(&sessionid.to_string());
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
        return HttpResponse::NotFound().body("not good");
    }

    if payload.title.len() < 3 {
        return HttpResponse::BadRequest().json(json!({"message": "标题过短"}));
    }

    let session_uuid = req.extensions().get::<Uuid>().unwrap().clone();
    let user = match get_session_by_session_id(&pool, session_uuid) {
        Ok(data) => data,
        Err(err) => {return HttpResponse::InternalServerError().json(json!({"message": err.to_string()}));},
    };
    
    let new_chat = NewChat::new(user.user_id, &payload.title);

    if let Err(err) = add_new_chat(&pool, &new_chat){
        return HttpResponse::InternalServerError().json(json!({"message": err.to_string()}))
    }else{
        HttpResponse::Ok().json(json!({"message": "创建成功"}))
    }
}