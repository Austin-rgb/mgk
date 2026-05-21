use super::db::Preferences;
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actixutils::Identity;
use serde::Deserialize;
use tracing::error; // Added for logging

use sqlx::{Pool, Sqlite};
#[derive(Clone)]
pub struct AppState {
    pub preferences: Preferences,
}

impl AppState {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        let preferences = Preferences::new(pool);
        Self { preferences }
    }
}

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PreferenceSetRequest {
    pub subject: String,
    pub address: String,
}

#[derive(Deserialize)]
pub struct PreferenceGetQuery {
    pub subject: String,
}

pub async fn set_preference(
    id: Identity,
    state: web::Data<AppState>,
    body: web::Json<PreferenceSetRequest>,
) -> impl Responder {
    match state
        .preferences
        .set(&id.sub.to_string(), &body.subject, body.address.clone())
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

pub async fn get_preference(
    id: Identity,
    state: web::Data<AppState>,
    query: web::Query<PreferenceGetQuery>,
) -> impl Responder {
    match state
        .preferences
        .get(&id.sub.to_string(), &query.subject)
        .await
    {
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
