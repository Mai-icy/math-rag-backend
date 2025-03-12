use bcrypt::{hash, DEFAULT_COST};
use chrono::{Local, NaiveDateTime};
use uuid::Uuid;

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn generate_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}