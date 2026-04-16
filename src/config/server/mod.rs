//! Server configuration modules
//!
//! Contains configuration specific to server mode (HTTP/gRPC services).
//! These configurations are only available when the `server` feature is enabled.

pub mod auth;
pub mod bootstrap;
pub mod connection_pool;
pub mod grpc;
pub mod http;
pub mod security;
pub mod telemetry;

pub use auth::*;
pub use bootstrap::*;
pub use connection_pool::*;
pub use grpc::*;
pub use http::*;
pub use security::*;
pub use telemetry::*;

use serde::{Deserialize, Serialize};

/// Server configuration aggregator
///
/// Contains all configuration specific to server mode.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ServerConfig {
    /// gRPC server configuration
    #[serde(default)]
    pub grpc: GrpcConfig,

    /// HTTP server configuration
    #[serde(default)]
    pub http: HttpServerConfig,

    /// Authentication configuration
    #[serde(default)]
    pub auth: AuthConfig,

    /// Bootstrap configuration
    #[serde(default)]
    pub bootstrap: BootstrapConfig,

    /// Telemetry configuration
    #[serde(default)]
    pub telemetry: TelemetryConfig,

    /// Connection pool configuration
    #[serde(default)]
    pub connection_pool: ConnectionPoolConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,
}

impl ServerConfig {
    /// Create a new server configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate all server configurations
    pub fn validate(&self) -> Result<(), String> {
        self.grpc.validate()?;
        self.http.validate()?;
        self.auth.validate()?;
        self.telemetry.validate()?;
        self.connection_pool.validate()?;
        self.security.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert!(config.grpc.enabled);
        assert!(config.http.enabled);
        assert!(config.auth.enable_authorize);
        assert!(config.telemetry.enabled);
    }

    #[test]
    fn test_server_config_validate() {
        let config = ServerConfig::default();
        assert!(config.validate().is_ok());
    }
}
