use bcrypt::{hash, DEFAULT_COST};
use chrono::{Local, NaiveDateTime};
use uuid::Uuid;
use jsonwebtoken::{encode, Header, EncodingKey, decode, DecodingKey, Validation, errors::Error, Algorithm};
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use std::{collections::HashSet, env};

#[derive(Serialize, Deserialize)]
struct Claims {
    session_id: String,
}

pub fn generate_jwt(session_id: &Uuid) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims {
        session_id: session_id.to_string(),
    };

    dotenv().ok();
    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let encoding_key = EncodingKey::from_secret(secret_key.as_ref());

    encode(&Header::default(), &claims, &encoding_key)
}

pub fn decode_jwt(token: &str) -> Result<Uuid, Error> {
    dotenv().ok();
    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");

    let decoding_key = DecodingKey::from_secret(secret_key.as_ref());
    
    let mut validation = Validation::new(Algorithm::HS256);
    validation.insecure_disable_signature_validation();
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new();

    let decoded = decode::<Claims>(token, &decoding_key, &validation)?;

    Uuid::parse_str(&decoded.claims.session_id)
        .map_err(|_| Error::from(jsonwebtoken::errors::ErrorKind::InvalidToken))
}

pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

pub fn generate_uuid() -> Uuid {
    Uuid::new_v4()
}

pub fn now() -> NaiveDateTime {
    Local::now().naive_local()
}