use anyhow::anyhow;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub cors_origin: String,
    pub host: String,
    pub port: u16,
    pub db_max_connections: u32,
    pub db_min_connections: u32,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_owned())
}

impl Config {
    /// Initialize configuration from environment variables and .env file
    pub fn init() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = match env::var("DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                let postgres_user = env_or("POSTGRES_USER", "postgres");
                let postgres_password = env_or("POSTGRES_PASSWORD", "password");
                let postgres_db = env_or("POSTGRES_DB", "backend_db");
                let postgres_host = env_or("POSTGRES_HOST", "localhost");
                let postgres_port = env_or("POSTGRES_PORT", "5432")
                    .parse::<u16>()
                    .map_err(|e| anyhow!("POSTGRES_PORT must be a valid u16: {}", e))?;
                format!(
                    "postgres://{}:{}@{}:{}/{}",
                    postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
                )
            }
        };

        let cors_origin = env_or("CORS_ORIGIN", "*");
        let host = env_or("HOST", "0.0.0.0");

        let port = env_or("PORT", "3000")
            .parse::<u16>()
            .map_err(|e| anyhow!("PORT must be a valid u16: {}", e))?;

        let db_max_connections = env_or("DB_MAX_CONNECTIONS", "100")
            .parse::<u32>()
            .map_err(|e| anyhow!("DB_MAX_CONNECTIONS must be a valid u32: {}", e))?;

        let db_min_connections = env_or("DB_MIN_CONNECTIONS", "5")
            .parse::<u32>()
            .map_err(|e| anyhow!("DB_MIN_CONNECTIONS must be a valid u32: {}", e))?;

        Ok(Config {
            database_url,
            cors_origin,
            host,
            port,
            db_max_connections,
            db_min_connections,
        })
    }
}
