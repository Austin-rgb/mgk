use super::db::Preferences;
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actixutils::Identity;
use serde::Deserialize;
use tracing::error; // Added for logging
use validator::Validate;

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Deserialize, Validate)]
pub struct PreferenceSetRequest {
    #[validate(length(max = 32))]
    pub subject: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct PreferenceGetQuery {
    pub subject: String,
}

#[derive(Deserialize, Validate)]
pub struct Token {
    #[validate(range(min = 100000, max = 999999))]
    pub token: u32,
}

pub async fn set_preference(
    id: Identity,
    state: web::Data<Preferences>,
    body: web::Json<PreferenceSetRequest>,
) -> impl Responder {
    match state
        .set(&id.sub.to_string(), &body.subject, &body.address)
        .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %id.sub.to_string(),
                subject = %body.subject,
                "Failed to set user preference"
            );
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub async fn confirm_preference(
    id: Identity,
    state: web::Data<Preferences>,
    body: web::Json<Token>,
) -> impl Responder {
    match state.confirm(&id.sub.to_string(), body.token).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %id.sub.to_string(),
                token = %body.token,
                "Failed to confirm user preference"
            );
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}

pub async fn get_preference(
    id: Identity,
    state: web::Data<Preferences>,
    query: web::Query<PreferenceGetQuery>,
) -> impl Responder {
    match state.get(&id.sub.to_string(), &query.subject).await {
        Ok(Some(channel)) => HttpResponse::Ok().json(channel),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %id.sub.to_string(),
                subject = %query.subject,
                "Failed to get user preference"
            );
            HttpResponse::InternalServerError().body(e.to_string())
        }
    }
}
