use anyhow::anyhow;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub cors_origin: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    /// Initialize configuration from environment variables and .env file
    pub fn init() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let postgres_user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let postgres_password =
            env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".to_string());
        let postgres_db = env::var("POSTGRES_DB").unwrap_or_else(|_| "backend_db".to_string());
        let postgres_host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
        let postgres_port = env::var("POSTGRES_PORT")
            .unwrap_or_else(|_| "5432".to_string())
            .parse::<u16>()
            .map_err(|e| anyhow!("POSTGRES_PORT must be a valid u16: {}", e))?;

        let database_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
        );

        let cors_origin = env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string());
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse::<u16>()
            .map_err(|e| anyhow!("PORT must be a valid u16: {}", e))?;

        Ok(Config {
            database_url,
            cors_origin,
            host,
            port,
        })
    }
}
