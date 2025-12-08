//! Optimizer module for optimizing execution plans
//! Contains the Optimizer implementation and various optimization rules

pub mod rule;
pub mod advanced_rules;
pub mod index_scan_rules;
pub mod join_rules;
pub mod limit_rules;
pub mod optimizer;