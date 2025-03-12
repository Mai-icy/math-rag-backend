use diesel::prelude::*;
use chrono::{NaiveDateTime, Duration};
use uuid::Uuid;
use serde::Deserialize;
use crate::schema::*;

use crate::utils::*;


#[derive(Queryable)]
pub struct User {
    user_id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    created_at: Option<NaiveDateTime>
}


#[derive(Queryable)]
pub struct Session {
    session_id: Uuid,
    user_id: Uuid,
    token: String,
    created_at: Option<NaiveDateTime>,
    expires_at: NaiveDateTime
}

#[derive(Queryable)]
pub struct Message{
    message_id: Uuid,
    chat_id: Uuid,
    role: String,
    content: String,
    timestamp: Option<NaiveDateTime>,
}

#[derive(Queryable)]
pub struct Chat{
    chat_id: Uuid,
    user_id: Uuid,
    title: String,
    created_at: Option<NaiveDateTime>,
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
    pub fn new(username: String, email: String, password: String) -> Self {
        let hashed_password = hash_password(&password).unwrap();
        Self {
            user_id: generate_uuid(),
            username,
            email,
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
    pub fn new(chatid: Uuid, userid: Uuid, title: &String) -> Self{
        Self{
            chat_id: chatid,
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
pub struct NewChatPayload {
    pub title: String,
}