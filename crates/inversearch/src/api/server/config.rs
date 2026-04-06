// Server configuration - only compiled when "service" feature is enabled
#![cfg(feature = "service")]

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 50051,
        }
    }
}

impl ServiceConfig {
    /// Create a new service configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let host = std::env::var("INVSEARCH_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("INVSEARCH_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(50051);

        Ok(Self { host, port })
    }
}

/// Server configuration (alias for ServiceConfig)
pub type ServerConfig = ServiceConfig;
