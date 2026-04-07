//! Optimization Pipeline Configuration
//!
//! This module provides configuration options for the optimization pipeline.

/// Configuration for the optimization pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Enable heuristic optimization phase
    pub enable_heuristic: bool,
    /// Enable cost-based optimization phase
    pub enable_cost_based: bool,
    /// Maximum iterations for heuristic rules
    pub max_heuristic_iterations: usize,
    /// Statistics threshold for optimization decisions
    pub statistics_threshold: u64,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            enable_heuristic: true,
            enable_cost_based: true,
            max_heuristic_iterations: 100,
            statistics_threshold: 1000,
        }
    }
}

impl PipelineConfig {
    /// Create a new pipeline configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a configuration that only enables heuristic optimization
    pub fn heuristic_only() -> Self {
        Self {
            enable_heuristic: true,
            enable_cost_based: false,
            ..Default::default()
        }
    }

    /// Create a configuration that only enables cost-based optimization
    pub fn cost_based_only() -> Self {
        Self {
            enable_heuristic: false,
            enable_cost_based: true,
            ..Default::default()
        }
    }

    /// Check if both optimization phases are enabled
    pub fn is_full_optimization(&self) -> bool {
        self.enable_heuristic && self.enable_cost_based
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PipelineConfig::default();
        assert!(config.enable_heuristic);
        assert!(config.enable_cost_based);
        assert!(config.is_full_optimization());
    }

    #[test]
    fn test_heuristic_only() {
        let config = PipelineConfig::heuristic_only();
        assert!(config.enable_heuristic);
        assert!(!config.enable_cost_based);
        assert!(!config.is_full_optimization());
    }

    #[test]
    fn test_cost_based_only() {
        let config = PipelineConfig::cost_based_only();
        assert!(!config.enable_heuristic);
        assert!(config.enable_cost_based);
        assert!(!config.is_full_optimization());
    }
}
