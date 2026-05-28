use crate::features::health::handler as health_handler;
use axum::{Router, routing::get};

use crate::routes::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(health_handler::health))
        .route("/ready", get(health_handler::readiness))
}
