use axum::{Router, routing::get};

use super::handler;
use crate::routes::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::health))
        .route("/ready", get(handler::readiness))
}
