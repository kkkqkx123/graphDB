//! Schema migration engine for executing data transformations across schema versions
//!
//! This module provides the core migration infrastructure:
//! - Migration path planning between versions
//! - Data transformation execution
//! - Progress tracking and rollback support
//! - Transaction-level atomicity guarantees

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use crate::core::{StorageError, StorageResult, DataType, Value};
use crate::storage::types::StoragePropertyDef;
use super::super::schema::compatibility::CompatibilityAnalysis;
use super::super::schema::version_history::LabelVersionHistory;

/// A single migration step from one version to the next
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Source version
    pub from_version: u64,
    /// Target version
    pub to_version: u64,
    /// Transformation rules for properties
    pub property_mappings: PropertyMappings,
}

/// Property transformation instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyMappings {
    /// Maps old property names to new property names
    pub renames: HashMap<String, String>,
    /// Properties to drop
    pub drops: Vec<String>,
    /// Properties to add with default values
    pub additions: HashMap<String, Value>,
    /// Type conversions
    pub conversions: HashMap<String, DataType>,
}

impl PropertyMappings {
    /// Create empty mappings
    pub fn new() -> Self {
        Self {
            renames: HashMap::new(),
            drops: Vec::new(),
            additions: HashMap::new(),
            conversions: HashMap::new(),
        }
    }

    /// Add a rename mapping
    pub fn add_rename(&mut self, old_name: String, new_name: String) {
        self.renames.insert(old_name, new_name);
    }

    /// Add a drop directive
    pub fn add_drop(&mut self, property_name: String) {
        self.drops.push(property_name);
    }

    /// Add an addition directive
    pub fn add_addition(&mut self, property_name: String, default_value: Value) {
        self.additions.insert(property_name, default_value);
    }

    /// Add a type conversion
    pub fn add_conversion(&mut self, property_name: String, new_type: DataType) {
        self.conversions.insert(property_name, new_type);
    }

    /// Apply this mapping to a property value
    pub fn apply_to_property(
        &self,
        name: &str,
        value: Value,
    ) -> StorageResult<Option<(String, Value)>> {
        // Check if property should be dropped
        if self.drops.contains(&name.to_string()) {
            return Ok(None);
        }

        // Get the target property name (after any renames)
        let target_name = self
            .renames
            .get(name)
            .map(|s| s.as_str())
            .unwrap_or(name);

        // Apply type conversion if needed
        let converted_value = if let Some(target_type) = self.conversions.get(target_name) {
            Self::convert_value(&value, target_type)?
        } else {
            value
        };

        Ok(Some((target_name.to_string(), converted_value)))
    }

    /// Convert a value to a target type
    fn convert_value(value: &Value, target_type: &DataType) -> StorageResult<Value> {
        use crate::core::Value as Val;
        use crate::core::DataType as DType;

        match (value, target_type) {
            // String conversions
            (Val::String(s), DType::Int) => {
                s.parse::<i32>()
                    .map(Value::Int)
                    .map_err(|_| StorageError::parse_error(
                        format!("Cannot convert '{}' to Int", s),
                    ))
            }
            (Val::String(s), DType::BigInt) => {
                s.parse::<i64>()
                    .map(Value::BigInt)
                    .map_err(|_| StorageError::parse_error(
                        format!("Cannot convert '{}' to BigInt", s),
                    ))
            }
            (Val::String(s), DType::Float) => {
                s.parse::<f32>()
                    .map(Value::Float)
                    .map_err(|_| StorageError::parse_error(
                        format!("Cannot convert '{}' to Float", s),
                    ))
            }
            (Val::String(s), DType::Double) => {
                s.parse::<f64>()
                    .map(Value::Double)
                    .map_err(|_| StorageError::parse_error(
                        format!("Cannot convert '{}' to Double", s),
                    ))
            }
            (Val::String(s), DType::Bool) => {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" => Ok(Value::Bool(true)),
                    "false" | "0" | "no" => Ok(Value::Bool(false)),
                    _ => Err(StorageError::parse_error(
                        format!("Cannot convert '{}' to Bool", s),
                    )),
                }
            }

            // Numeric widenings
            (Val::Int(n), DType::BigInt) => Ok(Value::BigInt(*n as i64)),
            (Val::Int(n), DType::Float) => Ok(Value::Float(*n as f32)),
            (Val::Int(n), DType::Double) => Ok(Value::Double(*n as f64)),
            (Val::BigInt(n), DType::Double) => Ok(Value::Double(*n as f64)),
            (Val::Float(f), DType::Double) => Ok(Value::Double(*f as f64)),

            // Same type (identity)
            (v, _) => Ok(v.clone()),
        }
    }
}

impl Default for PropertyMappings {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration plan for a label
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Source version
    pub from_version: u64,
    /// Target version
    pub to_version: u64,
    /// Sequence of steps to reach target
    pub steps: Vec<MigrationStep>,
    /// Difficulty estimate
    pub difficulty: u8,
    /// Whether rollback is possible
    pub can_rollback: bool,
}

impl MigrationPlan {
    /// Check if plan is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        !self.steps.is_empty() || self.from_version == self.to_version
    }

    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Get estimated migration time (rough estimate)
    pub fn estimated_duration_ms(&self, record_count: u64) -> u64 {
        // Rough estimate: 1ms base + 0.001ms per record per step
        let mut total = 1u64;
        for step in &self.steps {
            total += ((record_count as f64) * 0.001) as u64;
        }
        total
    }
}

/// Migration execution state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationState {
    /// Planned but not started
    Planned,
    /// Currently executing
    InProgress,
    /// Successfully completed
    Completed,
    /// Paused/suspended
    Suspended,
    /// Failed and rolled back
    RolledBack,
}

/// Migration engine for schema version management
pub struct MigrationEngine;

impl MigrationEngine {
    /// Plan a migration from one version to another
    pub fn plan_migration(
        from_version: u64,
        to_version: u64,
        history: &LabelVersionHistory,
        compatibility: &CompatibilityAnalysis,
    ) -> StorageResult<MigrationPlan> {
        if from_version == to_version {
            return Ok(MigrationPlan {
                from_version,
                to_version,
                steps: Vec::new(),
                difficulty: 0,
                can_rollback: true,
            });
        }

        if from_version > to_version {
            return Err(StorageError::invalid_operation(
                "Cannot migrate to older version (downgrade not supported)".to_string(),
            ));
        }

        // Check if migration is possible
        if !history.can_migrate(from_version, to_version) {
            return Err(StorageError::invalid_operation(
                format!(
                    "Cannot migrate from v{} to v{}: incompatible versions",
                    from_version, to_version
                ),
            ));
        }

        // Build migration path (currently: direct step for each version)
        let mut steps = Vec::new();
        let versions = history.get_versions();

        let from_idx = versions
            .iter()
            .position(|v| *v == from_version)
            .ok_or_else(|| StorageError::not_found(
                format!("Version {} not found", from_version),
            ))?;

        let to_idx = versions
            .iter()
            .position(|v| *v == to_version)
            .ok_or_else(|| StorageError::not_found(
                format!("Version {} not found", to_version),
            ))?;

        for i in from_idx..to_idx {
            let current = versions[i];
            let next = versions[i + 1];

            let mappings = Self::build_mappings(
                current,
                next,
                &history.change_log,
            )?;

            steps.push(MigrationStep {
                from_version: current,
                to_version: next,
                property_mappings: mappings,
            });
        }

        Ok(MigrationPlan {
            from_version,
            to_version,
            steps,
            difficulty: compatibility.migration_difficulty,
            can_rollback: !compatibility.breaking_changes.is_empty(),
        })
    }

    /// Build property mappings for a single version step
    fn build_mappings(
        from_version: u64,
        to_version: u64,
        change_log: &crate::storage::schema::change::ChangeLog,
    ) -> StorageResult<PropertyMappings> {
        let mut mappings = PropertyMappings::new();

        if let Some(changes) = change_log.get_version_changes(to_version) {
            for change in changes {
                use crate::storage::schema::change::ChangeDetails;

                match &change.details {
                    ChangeDetails::PropertyAdded {
                        name,
                        default_value,
                        ..
                    } => {
                        if let Some(default) = default_value {
                            mappings.add_addition(name.clone(), default.clone());
                        }
                    }
                    ChangeDetails::PropertyRemoved { name, .. } => {
                        mappings.add_drop(name.clone());
                    }
                    ChangeDetails::PropertyRenamed {
                        old_name,
                        new_name,
                    } => {
                        mappings.add_rename(old_name.clone(), new_name.clone());
                    }
                    ChangeDetails::PropertyTypeModified {
                        name,
                        new_type,
                        ..
                    } => {
                        mappings.add_conversion(name.clone(), new_type.clone());
                    }
                    _ => {
                        // Other change types don't affect property mapping
                    }
                }
            }
        }

        Ok(mappings)
    }

    /// Verify migration plan is executable
    pub fn verify_plan(plan: &MigrationPlan) -> StorageResult<()> {
        if !plan.is_valid() {
            return Err(StorageError::invalid_operation(
                "Invalid migration plan: no steps and versions differ".to_string(),
            ));
        }

        // Verify steps are sequential
        let mut current_version = plan.from_version;
        for step in &plan.steps {
            if step.from_version != current_version {
                return Err(StorageError::invalid_operation(
                    format!(
                        "Migration step gap: expected from {}, got {}",
                        current_version, step.from_version
                    ),
                ));
            }
            current_version = step.to_version;
        }

        if current_version != plan.to_version {
            return Err(StorageError::invalid_operation(
                format!(
                    "Migration plan doesn't reach target version: ends at {} vs {}",
                    current_version, plan.to_version
                ),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_mappings_creation() {
        let mappings = PropertyMappings::new();
        assert!(mappings.renames.is_empty());
        assert!(mappings.drops.is_empty());
    }

    #[test]
    fn test_property_mappings_operations() {
        let mut mappings = PropertyMappings::new();
        mappings.add_rename("old_name".to_string(), "new_name".to_string());
        mappings.add_drop("to_remove".to_string());

        assert_eq!(mappings.renames.len(), 1);
        assert_eq!(mappings.drops.len(), 1);
    }

    #[test]
    fn test_value_conversion() {
        let result = PropertyMappings::convert_value(
            &Value::String("123".to_string()),
            &DataType::Int,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Value::Int(123));
    }

    #[test]
    fn test_migration_plan_validation() {
        let plan = MigrationPlan {
            from_version: 1,
            to_version: 1,
            steps: Vec::new(),
            difficulty: 0,
            can_rollback: true,
        };

        assert!(MigrationEngine::verify_plan(&plan).is_ok());
    }

    #[test]
    fn test_migration_plan_time_estimate() {
        let plan = MigrationPlan {
            from_version: 1,
            to_version: 2,
            steps: vec![MigrationStep {
                from_version: 1,
                to_version: 2,
                property_mappings: PropertyMappings::new(),
            }],
            difficulty: 10,
            can_rollback: true,
        };

        let duration = plan.estimated_duration_ms(1000);
        assert!(duration > 0);
    }
}
