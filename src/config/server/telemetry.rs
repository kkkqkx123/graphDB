//! Telemetry configuration

use serde::{Deserialize, Serialize};

/// Telemetry configuration
///
/// Configures the telemetry server for metrics collection and monitoring.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TelemetryConfig {
    /// Whether to enable telemetry server
    pub enabled: bool,
    /// Bind address
    pub bind_address: String,
    /// Port number
    pub port: u16,
    /// Default output format (json or text)
    pub format: String,
    /// Maximum histogram entries before cleanup
    pub max_histogram_entries: usize,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            bind_address: "0.0.0.0".to_string(),
            port: 9090,
            format: "json".to_string(),
            max_histogram_entries: 10000,
            cleanup_interval_secs: 60,
        }
    }
}

impl TelemetryConfig {
    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Telemetry port cannot be 0".to_string());
        }

        if self.format.is_empty() {
            return Err("Telemetry format cannot be empty".to_string());
        }

        if self.max_histogram_entries == 0 {
            return Err("Max histogram entries must be greater than 0".to_string());
        }

        if self.cleanup_interval_secs == 0 {
            return Err("Cleanup interval must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.port, 9090);
        assert_eq!(config.bind_address, "0.0.0.0");
        assert_eq!(config.format, "json");
        assert_eq!(config.max_histogram_entries, 10000);
        assert_eq!(config.cleanup_interval_secs, 60);
    }

    #[test]
    fn test_telemetry_config_validate() {
        let config = TelemetryConfig::default();
        assert!(config.validate().is_ok());

        let invalid_config = TelemetryConfig {
            port: 0,
            ..Default::default()
        };
        assert!(invalid_config.validate().is_err());
    }
}
