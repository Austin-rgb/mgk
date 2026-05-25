use crate::prefs::db::{Preference, Token};

use super::db::Preferences;
use actix_web::web;
use actix_web::{HttpResponse, Responder};
use actixutils::Identity;
use serde::Deserialize;
use tracing::error; // Added for logging

// ---------------------------------------------------------------------------
// Preferences
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PreferenceGetQuery {
    pub subject: String,
}

pub async fn set_preference(
    id: Identity,
    state: web::Data<Preferences>,
    body: web::Json<Preference>,
) -> impl Responder {
    let pref = body.into_inner();
    match state.set(&id.sub.to_string(), pref.clone()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %id.sub.to_string(),
                subject = %pref.subject,
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
    let token = body.into_inner();
    match state.confirm(&id.sub.to_string(), &token).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            error!(
                error = %e,
                user = %id.sub.to_string(),
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
