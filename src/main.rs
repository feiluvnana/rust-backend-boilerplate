use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use rust_backend_boilerplate::{
    db::setup::connect_db,
    infra::config::Config,
    routes::{AppState, create_router},
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize configuration
    let config = Config::init()?;

    // Initialize tracing with EnvFilter
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,sqlx=warn".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Connect to Database & Run Migrations
    let db = connect_db(&config.database_url, 100, 5).await?;

    // Create AppState
    let state = AppState {
        db,
        config: config.clone(),
    };

    let allowed_origin = config
        .cors_origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("*"));

    // Setup routes and application layers
    let app = create_router(state)
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new())
                .on_request(DefaultOnRequest::new())
                .on_response(DefaultOnResponse::new()),
        )
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(RequestBodyLimitLayer::new(2_097_152)) // 2MB Limit
        .layer(
            CorsLayer::new()
                .allow_origin(allowed_origin)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    // Bind and serve with graceful shutdown
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for ctrl_c signal");
            info!("Graceful shutdown initiated");
        })
        .await?;

    Ok(())
}
