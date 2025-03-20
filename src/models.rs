use diesel::prelude::*;
use chrono::{NaiveDateTime, Duration};
use uuid::Uuid;
use serde::Deserialize;
use crate::schema::*;

use crate::utils::*;


#[derive(Queryable)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: Option<NaiveDateTime>
}


#[derive(Queryable, Clone)]
pub struct Session {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub created_at: Option<NaiveDateTime>,
    pub expires_at: NaiveDateTime
}

#[derive(Queryable)]
pub struct Message{
    pub message_id: Uuid,
    pub chat_id: Uuid,
    pub role: String,
    pub content: String,
    pub timestamp: Option<NaiveDateTime>,
}

#[derive(Queryable)]
pub struct Chat{
    pub chat_id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub created_at: Option<NaiveDateTime>,
}


#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    user_id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    created_at: Option<NaiveDateTime>
}

#[derive(Insertable)]
#[table_name="user_sessions"]
pub struct NewSession {
    session_id: Uuid,
    user_id: Uuid,
    token: String,
    created_at: Option<NaiveDateTime>,
    expires_at: Option<NaiveDateTime>
}

#[derive(Insertable)]
#[table_name="messages"]
pub struct NewMessage{
    message_id: Uuid,
    chat_id: Uuid,
    role: String,
    content: String,
    timestamp: Option<NaiveDateTime>,
}

#[derive(Insertable)]
#[table_name="chats"]
pub struct NewChat{
    chat_id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: Option<NaiveDateTime>,
}


impl NewUser {
    pub fn new(username: &String, email: &String, password: &String) -> Self {
        let hashed_password = hash_password(&password).unwrap();
        Self {
            user_id: generate_uuid(),
            username: username.clone(),
            email: email.clone(),
            password_hash: hashed_password,
            created_at: Some(now())
        }
    }
}

impl NewSession{
    pub fn new(sessionid: Uuid, userid: Uuid, token: &String) -> Self {
        Self{
            session_id: sessionid,
            user_id: userid,
            token: token.to_string(),
            created_at: Some(now()),
            expires_at: Some(now().checked_add_signed(Duration::days(1)).unwrap())
        }
    }
}

impl NewMessage{
    pub fn new(messageid: Uuid, chatid: Uuid, role: &String, content_: &String) -> Self{
        Self{
            message_id: messageid,
            chat_id: chatid,
            role: role.to_string(),
            content: content_.to_string(),
            timestamp: Some(now()),
        }
    }
}

impl NewChat{
    pub fn new(userid: Uuid, title: &String) -> Self{
        Self{
            chat_id: generate_uuid(),
            user_id: userid,
            title: title.to_string(),
            created_at: Some(now())
        }
    }
}


#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterPayload {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct NewChatPayload {
    pub title: String,
}