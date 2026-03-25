//! Actuator Factory Module
//!
//! Responsible for creating the corresponding executor instances based on the execution plan.
//! The design follows a modular approach, with responsibilities being divided into multiple sub-modules.
//! Parsers: These are programs responsible for interpreting various data elements such as vertex IDs, the direction of edges, and weight configurations.
//! Validators: Components responsible for verifying plan nodes, performing recursive checks, and ensuring security.
//! Constructors: These are responsible for creating various types of executors.
//! Executors: These are the components responsible for executing the execution plan.

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
