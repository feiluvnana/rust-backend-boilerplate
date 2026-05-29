use axum::{routing::get, Router};

use crate::routes::AppState;
use super::handler as handler;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(handler::health))
        .route("/ready", get(handler::readiness))
}
