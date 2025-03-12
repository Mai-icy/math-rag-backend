use diesel::prelude::*;
use chrono::NaiveDateTime;
use uuid::Uuid;
use crate::schema::*;


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
    expires_at: Option<NaiveDateTime>
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