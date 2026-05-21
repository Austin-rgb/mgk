use crate::prefs::handlers::*;
use actix_web::{middleware::from_fn, web};
use actixutils::middleware::authority;
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg
        // Preferences
        .route("/preferences/set", web::post().to(set_preference))
        .route("/preferences/get", web::get().to(get_preference));
}
