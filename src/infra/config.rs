use std::env;
use std::error::Error;

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
    pub fn init() -> Result<Self, Box<dyn Error>> {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL environment variable must be set")?;

        let cors_origin = env_or("CORS_ORIGIN", "*");
        let host = env_or("HOST", "0.0.0.0");

        let port = env_or("PORT", "3000")
            .parse::<u16>()
            .map_err(|e| format!("PORT must be a valid u16: {e}"))?;

        let db_max_connections = env_or("DB_MAX_CONNECTIONS", "100")
            .parse::<u32>()
            .map_err(|e| format!("DB_MAX_CONNECTIONS must be a valid u32: {e}"))?;

        let db_min_connections = env_or("DB_MIN_CONNECTIONS", "5")
            .parse::<u32>()
            .map_err(|e| format!("DB_MIN_CONNECTIONS must be a valid u32: {e}"))?;

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
