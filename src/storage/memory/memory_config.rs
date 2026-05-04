//! Memory Configuration
//!
//! Defines memory limits and allocation policies for the storage engine.

use std::fmt;

/// Default huge page size (2MB)
pub const DEFAULT_HUGE_PAGE_SIZE: usize = 2 * 1024 * 1024;

/// Minimum allocation size to consider using huge pages (1MB)
pub const DEFAULT_HUGE_PAGE_THRESHOLD: usize = 1024 * 1024;

/// Memory level for storage operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MemoryLevel {
    /// Pure in-memory storage, no disk persistence
    InMemory,
    /// In-memory with periodic sync to disk
    #[default]
    SyncToFile,
    /// Prefer huge pages for large allocations (Linux only)
    HugePagePreferred,
}

impl MemoryLevel {
    /// Check if this level prefers huge pages
    pub fn prefers_huge_pages(&self) -> bool {
        matches!(self, MemoryLevel::HugePagePreferred)
    }

    /// Check if this level requires disk persistence
    pub fn requires_persistence(&self) -> bool {
        matches!(self, MemoryLevel::SyncToFile)
    }

    /// Check if this is pure in-memory mode
    pub fn is_in_memory(&self) -> bool {
        matches!(self, MemoryLevel::InMemory)
    }
}

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
    /// Memory level for storage operations
    pub memory_level: MemoryLevel,
    /// Enable memory stalling when limit exceeded
    pub enable_stall: bool,
    /// Stall threshold ratio (0.0 - 1.0, when to start stalling)
    pub stall_threshold: f32,
    /// Huge page size in bytes (Linux only, default 2MB)
    pub huge_page_size: usize,
    /// Whether to fall back to regular pages if huge pages unavailable
    pub huge_page_fallback: bool,
    /// Minimum allocation size to use huge pages (default 1MB)
    pub huge_page_threshold: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_total_memory: 1024 * 1024 * 1024, // 1GB default
            vertex_memory_ratio: 0.4,
            edge_memory_ratio: 0.4,
            cache_memory_ratio: 0.2,
            memory_level: MemoryLevel::default(),
            enable_stall: true,
            stall_threshold: 0.9,
            huge_page_size: DEFAULT_HUGE_PAGE_SIZE,
            huge_page_fallback: true,
            huge_page_threshold: DEFAULT_HUGE_PAGE_THRESHOLD,
        }
    }
}

impl MemoryConfig {
    /// Create a new memory configuration with specified total memory
    pub fn with_total_memory(total: usize) -> Self {
        Self {
            max_total_memory: total,
            ..Default::default()
        }
    }

    /// Create a builder for custom configuration
    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), MemoryConfigError> {
        if self.max_total_memory == 0 {
            return Err(MemoryConfigError::InvalidTotalMemory(
                "Total memory must be greater than 0".to_string(),
            ));
        }

        let total_ratio =
            self.vertex_memory_ratio + self.edge_memory_ratio + self.cache_memory_ratio;

        if (total_ratio - 1.0).abs() > 0.001 {
            return Err(MemoryConfigError::InvalidRatio(format!(
                "Memory ratios must sum to 1.0, got {}",
                total_ratio
            )));
        }

        if self.vertex_memory_ratio < 0.0 || self.vertex_memory_ratio > 1.0 {
            return Err(MemoryConfigError::InvalidRatio(
                "Vertex memory ratio must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.edge_memory_ratio < 0.0 || self.edge_memory_ratio > 1.0 {
            return Err(MemoryConfigError::InvalidRatio(
                "Edge memory ratio must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.cache_memory_ratio < 0.0 || self.cache_memory_ratio > 1.0 {
            return Err(MemoryConfigError::InvalidRatio(
                "Cache memory ratio must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.stall_threshold < 0.0 || self.stall_threshold > 1.0 {
            return Err(MemoryConfigError::InvalidThreshold(
                "Stall threshold must be between 0.0 and 1.0".to_string(),
            ));
        }

        if self.huge_page_size == 0 || (self.huge_page_size & (self.huge_page_size - 1)) != 0 {
            return Err(MemoryConfigError::InvalidHugePageSize(
                "Huge page size must be a power of 2 and greater than 0".to_string(),
            ));
        }

        if self.huge_page_threshold == 0 {
            return Err(MemoryConfigError::InvalidHugePageThreshold(
                "Huge page threshold must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Get maximum memory for vertex data
    pub fn max_vertex_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.vertex_memory_ratio as f64) as usize
    }

    /// Get maximum memory for edge data
    pub fn max_edge_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.edge_memory_ratio as f64) as usize
    }

    /// Get maximum memory for cache
    pub fn max_cache_memory(&self) -> usize {
        (self.max_total_memory as f64 * self.cache_memory_ratio as f64) as usize
    }

    /// Get stall threshold in bytes
    pub fn stall_threshold_bytes(&self) -> usize {
        (self.max_total_memory as f64 * self.stall_threshold as f64) as usize
    }

    /// Check if the configuration enables stalling
    pub fn is_stall_enabled(&self) -> bool {
        self.enable_stall
    }

    /// Check if huge pages should be used for an allocation of the given size
    pub fn should_use_huge_pages(&self, allocation_size: usize) -> bool {
        self.memory_level.prefers_huge_pages() && allocation_size >= self.huge_page_threshold
    }

    /// Get the huge page size
    pub fn get_huge_page_size(&self) -> usize {
        self.huge_page_size
    }

    /// Check if huge page fallback is enabled
    pub fn is_huge_page_fallback_enabled(&self) -> bool {
        self.huge_page_fallback
    }

    /// Align a size to the huge page boundary
    pub fn align_to_huge_page(&self, size: usize) -> usize {
        let mask = self.huge_page_size - 1;
        (size + mask) & !mask
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

    pub fn memory_level(mut self, level: MemoryLevel) -> Self {
        self.config.memory_level = level;
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

    pub fn huge_page_size(mut self, size: usize) -> Self {
        self.config.huge_page_size = size;
        self
    }

    pub fn huge_page_fallback(mut self, fallback: bool) -> Self {
        self.config.huge_page_fallback = fallback;
        self
    }

    pub fn huge_page_threshold(mut self, threshold: usize) -> Self {
        self.config.huge_page_threshold = threshold;
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
    /// Invalid total memory value
    InvalidTotalMemory(String),
    /// Invalid memory ratio
    InvalidRatio(String),
    /// Invalid stall threshold
    InvalidThreshold(String),
    /// Invalid huge page size
    InvalidHugePageSize(String),
    /// Invalid huge page threshold
    InvalidHugePageThreshold(String),
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
            MemoryConfigError::InvalidHugePageSize(msg) => {
                write!(f, "Invalid huge page size: {}", msg)
            }
            MemoryConfigError::InvalidHugePageThreshold(msg) => {
                write!(f, "Invalid huge page threshold: {}", msg)
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
        assert_eq!(config.huge_page_size, DEFAULT_HUGE_PAGE_SIZE);
        assert!(config.huge_page_fallback);
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

    #[test]
    fn test_memory_level_helpers() {
        assert!(MemoryLevel::HugePagePreferred.prefers_huge_pages());
        assert!(!MemoryLevel::InMemory.prefers_huge_pages());
        assert!(!MemoryLevel::SyncToFile.prefers_huge_pages());

        assert!(MemoryLevel::SyncToFile.requires_persistence());
        assert!(!MemoryLevel::InMemory.requires_persistence());

        assert!(MemoryLevel::InMemory.is_in_memory());
        assert!(!MemoryLevel::SyncToFile.is_in_memory());
    }

    #[test]
    fn test_huge_page_config() {
        let config = MemoryConfigBuilder::default()
            .memory_level(MemoryLevel::HugePagePreferred)
            .huge_page_size(1024 * 1024)
            .huge_page_fallback(false)
            .huge_page_threshold(512 * 1024)
            .build()
            .unwrap();

        assert!(config.should_use_huge_pages(1024 * 1024));
        assert!(!config.should_use_huge_pages(256 * 1024));
        assert_eq!(config.get_huge_page_size(), 1024 * 1024);
        assert!(!config.is_huge_page_fallback_enabled());
    }

    #[test]
    fn test_huge_page_alignment() {
        let config = MemoryConfig::default();

        assert_eq!(config.align_to_huge_page(1), DEFAULT_HUGE_PAGE_SIZE);
        assert_eq!(
            config.align_to_huge_page(DEFAULT_HUGE_PAGE_SIZE),
            DEFAULT_HUGE_PAGE_SIZE
        );
        assert_eq!(
            config.align_to_huge_page(DEFAULT_HUGE_PAGE_SIZE + 1),
            DEFAULT_HUGE_PAGE_SIZE * 2
        );
    }

    #[test]
    fn test_invalid_huge_page_size() {
        let config = MemoryConfigBuilder::default()
            .huge_page_size(0)
            .build();
        assert!(config.is_err());

        let config = MemoryConfigBuilder::default()
            .huge_page_size(1000)
            .build();
        assert!(config.is_err());
    }

    #[test]
    fn test_invalid_huge_page_threshold() {
        let config = MemoryConfigBuilder::default()
            .huge_page_threshold(0)
            .build();
        assert!(config.is_err());
    }
}
