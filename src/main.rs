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

    // Connect to Database
    let db = connect_db(
        &config.database_url,
        config.db_max_connections,
        config.db_min_connections,
    )
    .await?;

    // Create AppState
    let state = AppState {
        db,
        config: config.clone(),
    };

    // Setup routes and application layers
    let app = create_router(state);

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
