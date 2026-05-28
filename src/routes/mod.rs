use crate::{infra::config::Config, middleware::request_id::request_id_middleware};
use axum::{Router, extract::FromRef, http::HeaderValue, middleware::from_fn};
use sea_orm::DatabaseConnection;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

pub mod health;
pub mod swagger;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Config,
}

impl FromRef<AppState> for DatabaseConnection {
    fn from_ref(state: &AppState) -> Self {
        state.db.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

pub fn create_router(state: AppState) -> Router {
    let allowed_origin = state
        .config
        .cors_origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("*"));

    let api_routes = Router::new()
        .nest("/health", health::router())
        .layer(from_fn(request_id_middleware));

    // Security headers (nosniff, DENY, 1; mode=block)
    let security_headers = [
        (axum::http::header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (axum::http::header::X_FRAME_OPTIONS, "DENY"),
        (axum::http::header::X_XSS_PROTECTION, "1; mode=block"),
    ];

    let mut api_routes = api_routes;
    for (header, val) in security_headers {
        api_routes = api_routes.layer(SetResponseHeaderLayer::overriding(
            header,
            HeaderValue::from_static(val),
        ));
    }

    Router::new()
        .merge(
            SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", swagger::ApiDoc::openapi()),
        )
        .nest("/api", api_routes)
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(2_097_152)) // 2MB Limit
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}
