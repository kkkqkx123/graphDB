use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineType {
    Qdrant,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorClientConfig {
    pub enabled: bool,
    pub engine: EngineType,
    pub connection: ConnectionConfig,
    pub timeout: TimeoutConfig,
    pub retry: RetryConfig,
}

impl VectorClientConfig {
    pub fn new(engine: EngineType) -> Self {
        Self {
            enabled: true,
            engine,
            connection: ConnectionConfig::default(),
            timeout: TimeoutConfig::default(),
            retry: RetryConfig::default(),
        }
    }

    pub fn qdrant() -> Self {
        Self::new(EngineType::Qdrant)
    }

    pub fn qdrant_local(host: &str, grpc_port: u16, http_port: u16) -> Self {
        Self {
            enabled: true,
            engine: EngineType::Qdrant,
            connection: ConnectionConfig {
                host: host.to_string(),
                port: grpc_port,
                use_tls: false,
                api_key: None,
                connect_timeout_secs: 5,
                http_port: Some(http_port),
            },
            timeout: TimeoutConfig::default(),
            retry: RetryConfig::default(),
        }
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            engine: EngineType::Qdrant,
            connection: ConnectionConfig::default(),
            timeout: TimeoutConfig::default(),
            retry: RetryConfig::default(),
        }
    }

    pub fn with_connection(mut self, connection: ConnectionConfig) -> Self {
        self.connection = connection;
        self
    }

    pub fn with_timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    pub fn to_qdrant_config(&self) -> VectorClientConfig {
        VectorClientConfig {
            enabled: self.enabled,
            engine: EngineType::Qdrant,
            connection: self.connection.clone(),
            timeout: self.timeout.clone(),
            retry: self.retry.clone(),
        }
    }
}

impl Default for VectorClientConfig {
    fn default() -> Self {
        Self::qdrant()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub api_key: Option<String>,
    pub connect_timeout_secs: u64,
    pub http_port: Option<u16>,
}

impl ConnectionConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            use_tls: false,
            api_key: None,
            connect_timeout_secs: 5,
            http_port: None,
        }
    }

    pub fn localhost(port: u16) -> Self {
        Self::new("localhost", port)
    }

    pub fn qdrant_local(grpc_port: u16, http_port: u16) -> Self {
        Self {
            host: "localhost".to_string(),
            port: grpc_port,
            use_tls: false,
            api_key: None,
            connect_timeout_secs: 5,
            http_port: Some(http_port),
        }
    }

    pub fn with_tls(mut self) -> Self {
        self.use_tls = true;
        self
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn to_url(&self) -> String {
        let scheme = if self.use_tls { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    pub fn to_grpc_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self::localhost(6333)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub request_timeout_secs: u64,
    pub search_timeout_secs: u64,
    pub upsert_timeout_secs: u64,
}

impl TimeoutConfig {
    pub fn new(request: u64, search: u64, upsert: u64) -> Self {
        Self {
            request_timeout_secs: request,
            search_timeout_secs: search,
            upsert_timeout_secs: upsert,
        }
    }

    pub fn request_duration(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub fn search_duration(&self) -> Duration {
        Duration::from_secs(self.search_timeout_secs)
    }

    pub fn upsert_duration(&self) -> Duration {
        Duration::from_secs(self.upsert_timeout_secs)
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self::new(30, 60, 30)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub multiplier: f64,
}

impl RetryConfig {
    pub fn new(
        max_retries: usize,
        initial_delay_ms: u64,
        max_delay_ms: u64,
        multiplier: f64,
    ) -> Self {
        Self {
            max_retries,
            initial_delay_ms,
            max_delay_ms,
            multiplier,
        }
    }

    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_delay_ms: 0,
            max_delay_ms: 0,
            multiplier: 1.0,
        }
    }

    pub fn calculate_delay(&self, attempt: usize) -> Duration {
        if attempt == 0 {
            return Duration::from_millis(0);
        }
        let delay = self.initial_delay_ms as f64 * self.multiplier.powi(attempt as i32 - 1);
        let delay = delay.min(self.max_delay_ms as f64) as u64;
        Duration::from_millis(delay)
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new(3, 100, 5000, 2.0)
    }
}

// Simple validation methods
impl VectorClientConfig {
    /// Validate configuration
    ///
    /// Returns Ok(()) if valid, Err(message) if invalid
    pub fn validate(&self) -> Result<(), String> {
        self.connection.validate()?;
        self.timeout.validate()?;
        self.retry.validate()?;
        Ok(())
    }
}

impl ConnectionConfig {
    /// Validate connection configuration
    /// u16 can't be greater than 65535, so only check 0
    pub fn validate(&self) -> Result<(), String> {
        if self.host.is_empty() {
            return Err("connection.host cannot be empty".to_string());
        }
        if self.port == 0 {
            return Err("connection.port must not be 0".to_string());
        }
        if self.connect_timeout_secs == 0 {
            return Err("connection.connect_timeout_secs must be greater than 0".to_string());
        }
        if self.http_port == Some(0) {
            return Err("connection.http_port must not be 0".to_string());
        }
        Ok(())
    }
}

impl TimeoutConfig {
    /// Validate timeout configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.request_timeout_secs == 0 {
            return Err("timeout.request_timeout_secs must be greater than 0".to_string());
        }
        if self.search_timeout_secs == 0 {
            return Err("timeout.search_timeout_secs must be greater than 0".to_string());
        }
        if self.upsert_timeout_secs == 0 {
            return Err("timeout.upsert_timeout_secs must be greater than 0".to_string());
        }
        Ok(())
    }
}

impl RetryConfig {
    /// Validate retry configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_retries > 10 {
            return Err("retry.max_retries must not exceed 10".to_string());
        }
        if self.multiplier < 1.0 || self.multiplier > 10.0 {
            return Err("retry.multiplier must be in range [1.0, 10.0]".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_connection_config_valid() {
        let config = ConnectionConfig::new("localhost", 6333);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_connection_config_empty_host() {
        let config = ConnectionConfig::new("", 6333);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_config_invalid_port() {
        let config = ConnectionConfig::new("localhost", 0);
        assert!(config.validate().is_err());

        let config = ConnectionConfig::new("localhost", 65536);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_timeout_config_valid() {
        let config = TimeoutConfig::new(30, 60, 30);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_timeout_config_zero() {
        let config = TimeoutConfig::new(0, 60, 30);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_retry_config_valid() {
        let config = RetryConfig::new(3, 100, 5000, 2.0);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_retry_config_invalid_multiplier() {
        let config = RetryConfig::new(3, 100, 5000, 0.5);
        assert!(config.validate().is_err());

        let config = RetryConfig::new(3, 100, 5000, 15.0);
        assert!(config.validate().is_err());
    }
}
