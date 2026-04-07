//! Optimization Phase Definitions
//!
//! This module defines the different phases of the optimization pipeline.

/// Optimization phases in the pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptimizationPhase {
    /// Phase 1: Heuristic Optimization
    /// Always executed, applies rule-based transformations
    Heuristic,

    /// Phase 2: Cost-Based Optimization
    /// Optional, uses statistics and cost models for decision-making
    CostBased,
}

impl OptimizationPhase {
    /// Get the name of the phase
    pub fn name(&self) -> &'static str {
        match self {
            OptimizationPhase::Heuristic => "Heuristic",
            OptimizationPhase::CostBased => "CostBased",
        }
    }

    /// Check if this phase is always executed
    pub fn is_mandatory(&self) -> bool {
        match self {
            OptimizationPhase::Heuristic => true,
            OptimizationPhase::CostBased => false,
        }
    }

    /// Get all phases in execution order
    pub fn all_phases() -> Vec<OptimizationPhase> {
        vec![OptimizationPhase::Heuristic, OptimizationPhase::CostBased]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_names() {
        assert_eq!(OptimizationPhase::Heuristic.name(), "Heuristic");
        assert_eq!(OptimizationPhase::CostBased.name(), "CostBased");
    }

    #[test]
    fn test_phase_mandatory() {
        assert!(OptimizationPhase::Heuristic.is_mandatory());
        assert!(!OptimizationPhase::CostBased.is_mandatory());
    }

    #[test]
    fn test_all_phases() {
        let phases = OptimizationPhase::all_phases();
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0], OptimizationPhase::Heuristic);
        assert_eq!(phases[1], OptimizationPhase::CostBased);
    }
}
