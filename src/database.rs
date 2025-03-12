use crate::models::{NewChat, NewSession, NewUser, NewMessage, User, Chat, Session, Message};
use crate::schema::chats;
use crate::schema::users;
use crate::schema::messages;
use crate::schema::user_sessions;


use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::env;
use dotenv::dotenv;
use uuid::Uuid;
use chrono::NaiveDateTime;


pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn init_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .build(manager)
        .expect("数据库连接池初始化失败")
}


pub fn get_user_by_username(pool: &DbPool, user_name: &String) -> Result<User, diesel::result::Error> {
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;
    
    users::table.filter(users::username.eq(user_name))
        .first::<User>(&mut conn)
}

pub fn add_new_user(pool: &DbPool, new_user: &NewUser) -> Result<usize, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;
    
    diesel::insert_into(users::table)
        .values(new_user)
        .execute(&mut conn)
}

pub fn add_new_session(pool: &DbPool, new_session: &NewSession) -> Result<usize, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    diesel::insert_into(user_sessions::table)
        .values(new_session)
        .execute(&mut conn)
}

pub fn get_session_by_session_id(pool: &DbPool, session_uuid: Uuid) -> Result<Session, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    user_sessions::table.filter(user_sessions::session_id.eq(session_uuid))
        .first::<Session>(&mut conn)
}

pub fn update_expries_by_session(pool: &DbPool, session_uuid: Uuid, expries_time: NaiveDateTime) -> Result<usize, diesel::result::Error> {
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    diesel::update(user_sessions::table.filter(user_sessions::session_id.eq(session_uuid)))
        .set(user_sessions::expires_at.eq(expries_time)) 
        .execute(&mut conn)
}

pub fn add_new_chat(pool: &DbPool, new_chat: &NewChat) -> Result<usize, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    diesel::insert_into(chats::table)
        .values(new_chat)
        .execute(&mut conn)
}

pub fn get_all_chats_by_user_id(pool: &DbPool, userid: Uuid) -> Result<Vec<Chat>, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    chats::table.filter(chats::user_id.eq(userid))
        .load::<Chat>(&mut conn)
}

pub fn add_new_message(pool: &DbPool, new_message: &NewMessage) -> Result<usize, diesel::result::Error>{
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;

    diesel::insert_into(messages::table)
        .values(new_message)
        .execute(&mut conn)
}

pub fn get_all_messages_by_chat_id(pool: &DbPool, chatid: Uuid) -> Result<Vec<Message>, diesel::result::Error> {
    let mut conn = pool.get().map_err(|_| diesel::result::Error::DatabaseError(
        diesel::result::DatabaseErrorKind::UnableToSendCommand,
        Box::new("Failed to get DB connection".to_string()),
    ))?;
    
    messages::table.filter(messages::chat_id.eq(chatid))
        .load::<Message>(&mut conn)
}