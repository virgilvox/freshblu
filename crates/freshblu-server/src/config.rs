use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP port
    pub http_port: u16,
    /// MQTT port
    pub mqtt_port: u16,
    /// Database URL (sqlite:freshblu.db or postgresql://...)
    pub database_url: String,
    /// Bcrypt pepper for extra token security
    pub pepper: String,
    /// Open registration (no auth required to register)
    pub open_registration: bool,
    /// Max message size in bytes
    pub max_message_size: usize,
    /// Log level
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 3000,
            mqtt_port: 1883,
            database_url: "sqlite:freshblu.db".to_string(),
            pepper: "change-me-in-production".to_string(),
            open_registration: true,
            max_message_size: 1_048_576, // 1MB
            log_level: "info".to_string(),
        }
    }
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();
        Self {
            http_port: std::env::var("FRESHBLU_HTTP_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3000),
            mqtt_port: std::env::var("FRESHBLU_MQTT_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1883),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite:freshblu.db".to_string()),
            pepper: std::env::var("FRESHBLU_PEPPER")
                .unwrap_or_else(|_| "change-me-in-production".to_string()),
            open_registration: std::env::var("FRESHBLU_OPEN_REGISTRATION")
                .map(|v| v != "false")
                .unwrap_or(true),
            max_message_size: std::env::var("FRESHBLU_MAX_MESSAGE_SIZE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1_048_576),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
        }
    }
}
