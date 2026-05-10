//! Memory Configuration
//!
//! Defines memory limits and allocation policies for the storage engine.

use std::fmt;

/// Memory configuration for the storage engine
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Maximum total memory in bytes
    pub max_total_memory: usize,
    /// Ratio of memory allocated for vertex data (0.0 - 1.0)
    pub vertex_memory_ratio: f32,
    /// Ratio of memory allocated for edge data (0.0 - 1.0)
    pub edge_memory_ratio: f32,
    /// Ratio of memory allocated for cache (0.0 - 1.0)
    pub cache_memory_ratio: f32,
    /// Enable memory stalling when limit exceeded
    pub enable_stall: bool,
    /// Stall threshold ratio (0.0 - 1.0, when to start stalling)
    pub stall_threshold: f32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_total_memory: 1024 * 1024 * 1024,
            vertex_memory_ratio: 0.4,
            edge_memory_ratio: 0.4,
            cache_memory_ratio: 0.2,
            enable_stall: true,
            stall_threshold: 0.9,
        }
    }
}

impl MemoryConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_total_memory(total: usize) -> Self {
        Self {
            max_total_memory: total,
            ..Default::default()
        }
    }

    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::default()
    }

    pub fn validate(&self) -> Result<(), MemoryConfigError> {
        if self.max_total_memory == 0 {
            return Err(MemoryConfigError::InvalidTotalMemory(
                "Total memory must be greater than 0".to_string(),
            ));
        }

        let ratio_sum = self.vertex_memory_ratio + self.edge_memory_ratio + self.cache_memory_ratio;
        if (ratio_sum - 1.0).abs() > 0.01 {
            return Err(MemoryConfigError::InvalidRatio(
                "Memory ratios must sum to 1.0".to_string(),
            ));
        }

        if self.stall_threshold < 0.0 || self.stall_threshold > 1.0 {
            return Err(MemoryConfigError::InvalidThreshold(
                "Stall threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(())
    }

    pub fn max_vertex_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.vertex_memory_ratio as f64) as usize
    }

    pub fn max_edge_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.edge_memory_ratio as f64) as usize
    }

    pub fn max_cache_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.cache_memory_ratio as f64) as usize
    }

    pub fn stall_threshold_bytes(&self) -> usize {
        (self.max_total_memory as f64 * self.stall_threshold as f64) as usize
    }

    pub fn is_stall_enabled(&self) -> bool {
        self.enable_stall
    }
}

/// Builder for MemoryConfig
#[derive(Default)]
pub struct MemoryConfigBuilder {
    config: MemoryConfig,
}

impl MemoryConfigBuilder {
    pub fn total_memory(mut self, bytes: usize) -> Self {
        self.config.max_total_memory = bytes;
        self
    }

    pub fn vertex_ratio(mut self, ratio: f32) -> Self {
        self.config.vertex_memory_ratio = ratio;
        self
    }

    pub fn edge_ratio(mut self, ratio: f32) -> Self {
        self.config.edge_memory_ratio = ratio;
        self
    }

    pub fn cache_ratio(mut self, ratio: f32) -> Self {
        self.config.cache_memory_ratio = ratio;
        self
    }

    pub fn enable_stall(mut self, enable: bool) -> Self {
        self.config.enable_stall = enable;
        self
    }

    pub fn stall_threshold(mut self, threshold: f32) -> Self {
        self.config.stall_threshold = threshold;
        self
    }

    pub fn build(self) -> Result<MemoryConfig, MemoryConfigError> {
        self.config.validate()?;
        Ok(self.config)
    }
}

/// Errors that can occur during memory configuration
#[derive(Debug, Clone)]
pub enum MemoryConfigError {
    InvalidTotalMemory(String),
    InvalidRatio(String),
    InvalidThreshold(String),
}

impl fmt::Display for MemoryConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryConfigError::InvalidTotalMemory(msg) => {
                write!(f, "Invalid total memory: {}", msg)
            }
            MemoryConfigError::InvalidRatio(msg) => write!(f, "Invalid ratio: {}", msg),
            MemoryConfigError::InvalidThreshold(msg) => {
                write!(f, "Invalid threshold: {}", msg)
            }
        }
    }
}

impl std::error::Error for MemoryConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = MemoryConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.max_total_memory, 1024 * 1024 * 1024);
        assert_eq!(config.vertex_memory_ratio, 0.4);
        assert_eq!(config.edge_memory_ratio, 0.4);
        assert_eq!(config.cache_memory_ratio, 0.2);
    }

    #[test]
    fn test_memory_limits() {
        let config = MemoryConfig::with_total_memory(1000);
        assert_eq!(config.max_vertex_memory(), 400);
        assert_eq!(config.max_edge_memory(), 400);
        assert_eq!(config.max_cache_memory(), 200);
    }

    #[test]
    fn test_invalid_ratio() {
        let config = MemoryConfigBuilder::default()
            .vertex_ratio(0.5)
            .edge_ratio(0.6)
            .cache_ratio(0.1)
            .build();
        assert!(config.is_err());
    }

    #[test]
    fn test_builder() {
        let config = MemoryConfigBuilder::default()
            .total_memory(2 * 1024 * 1024 * 1024)
            .vertex_ratio(0.5)
            .edge_ratio(0.3)
            .cache_ratio(0.2)
            .build()
            .unwrap();

        assert_eq!(config.max_total_memory, 2 * 1024 * 1024 * 1024);
        assert_eq!(config.max_vertex_memory(), 1024 * 1024 * 1024);
    }
}
