//! Actuator Factory Module
//!
//! Responsible for creating the corresponding executor instances based on the execution plan.
//!
//! ## Module Structure
//!
//! - `parsers`: Parse vertex IDs, edge directions, weight configurations
//! - `validators`: Verify plan nodes, recursive checks, security validation
//! - `builders`: Builder structs for each executor category
//! - `executor_factory`: Main factory coordinating creation
//! - `executors`: Plan execution components

pub mod builders;
pub mod executor_factory;
pub mod executors;
pub mod parsers;
pub mod validators;

// Re-export the main types
pub use executor_factory::ExecutorFactory;
pub use executors::PlanExecutor;
pub use validators::{RecursionDetector, SafetyValidator};

// Re-export the security configuration from the sub-module.
pub use validators::safety_validator::ExecutorSafetyConfig;
