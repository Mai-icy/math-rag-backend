use actix_web::middleware::Next;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{web, Error, HttpMessage};
use actix_web::body::MessageBody;
use chrono::Utc;
use crate::utils::decode_jwt;
use crate::database::{get_session_by_session_id, DbPool};

// 统一的失败处理
async fn fail_auth<B>(req: ServiceRequest, next: Next<B>) -> Result<ServiceResponse<B>, Error>
where
    B: MessageBody,
{
    req.extensions_mut().insert(false);
    Ok(next.call(req).await?)
}

pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let auth_header = req.headers().get("Authorization");

    let token = match auth_header.and_then(|h| h.to_str().ok()) {
        Some(t) => t.to_string(),
        None => return fail_auth(req, next).await,
    };

    let session_uuid = match decode_jwt(&token) {
        Ok(uuid) => uuid,
        Err(_) => return fail_auth(req, next).await,
    };

    let pool = match req.app_data::<web::Data<DbPool>>() {
        Some(pool) => pool,
        None => return fail_auth(req, next).await,
    };

    let session = match get_session_by_session_id(pool, session_uuid) {
        Ok(session) => session,
        Err(_) => return fail_auth(req, next).await,
    };

    if session.token != token {
        return fail_auth(req, next).await;
    }

    if session.expires_at < Utc::now().naive_utc() {
        return fail_auth(req, next).await;
    }


    req.extensions_mut().insert(true);
    req.extensions_mut().insert(session);

    next.call(req).await
}