//! Schema compatibility analysis
//!
//! Analyzes schema changes to identify breaking and non-breaking modifications,
//! assigning compatibility scores for migration planning.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::core::StorageResult;
use super::change::{ChangeDetails, SchemaChange};
use crate::storage::types::StoragePropertyDef;

/// Type of breaking change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingChangeType {
    /// A property was removed completely
    PropertyRemoved,
    /// A property's type was changed (incompatible)
    PropertyTypeChanged,
    /// The primary key was changed
    PrimaryKeyRemoved,
}

/// Type of non-breaking change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NonBreakingChangeType {
    /// A new optional property was added (nullable)
    OptionalPropertyAdded,
    /// A new property with default value was added
    DefaultedPropertyAdded,
    /// A property was renamed (backward compatible if tracked)
    PropertyRenamed,
    /// Property became nullable (more permissive)
    PropertyBecameNullable,
}

/// Breaking change details with information for migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    pub change_type: BreakingChangeType,
    pub property_name: Option<String>,
    pub description: String,
}

/// Non-breaking change details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonBreakingChange {
    pub change_type: NonBreakingChangeType,
    pub property_name: Option<String>,
    pub description: String,
}

/// Compatibility analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityAnalysis {
    /// Whether the new schema is compatible with the old schema
    pub is_compatible: bool,
    /// Compatibility score (0-100): higher means fewer breaking changes
    pub compatibility_score: u8,
    /// All breaking changes detected
    pub breaking_changes: Vec<BreakingChange>,
    /// All non-breaking changes detected
    pub non_breaking_changes: Vec<NonBreakingChange>,
    /// Migration difficulty estimate: 0 (trivial) to 100 (impossible)
    pub migration_difficulty: u8,
    /// Migration strategy recommendation
    pub migration_strategy: MigrationStrategy,
}

/// Migration strategy based on compatibility analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    /// No migration needed (forward-compatible)
    NoMigration,
    /// Simple data mapping (property renames, defaults)
    SimpleMapping,
    /// Complex data transformation (type conversions)
    ComplexTransformation,
    /// Manual intervention required (incompatible changes)
    ManualIntervention,
}

impl CompatibilityAnalysis {
    /// Create a new compatibility analysis
    pub fn new() -> Self {
        Self {
            is_compatible: true,
            compatibility_score: 100,
            breaking_changes: Vec::new(),
            non_breaking_changes: Vec::new(),
            migration_difficulty: 0,
            migration_strategy: MigrationStrategy::NoMigration,
        }
    }

    /// Add a breaking change
    pub fn add_breaking_change(&mut self, change: BreakingChange) {
        self.is_compatible = false;
        self.breaking_changes.push(change);
        self.update_scores();
    }

    /// Add a non-breaking change
    pub fn add_non_breaking_change(&mut self, change: NonBreakingChange) {
        self.non_breaking_changes.push(change);
        self.update_scores();
    }

    /// Update compatibility scores based on detected changes
    fn update_scores(&mut self) {
        // Calculate compatibility score (0-100)
        // Base: 100 points
        // Each breaking change: -20 points
        // Each non-breaking change: -2 points
        let breaking_penalty = (self.breaking_changes.len() as u8).saturating_mul(20);
        let non_breaking_penalty = (self.non_breaking_changes.len() as u8).saturating_mul(2);

        self.compatibility_score = (100u16)
            .saturating_sub(breaking_penalty as u16)
            .saturating_sub(non_breaking_penalty as u16) as u8;

        // Update migration difficulty (inverse of compatibility)
        self.migration_difficulty = if self.breaking_changes.is_empty() {
            ((self.non_breaking_changes.len() as u8) * 5).min(30)
        } else {
            100 - self.compatibility_score
        };

        // Determine migration strategy
        self.migration_strategy = if self.breaking_changes.is_empty() {
            if self.non_breaking_changes.is_empty() {
                MigrationStrategy::NoMigration
            } else {
                MigrationStrategy::SimpleMapping
            }
        } else {
            // For now, all breaking changes require manual intervention
            MigrationStrategy::ManualIntervention
        };
    }

    /// Get human-readable summary
    pub fn summary(&self) -> String {
        format!(
            "Compatibility: {} (score: {}/100, difficulty: {})\n  Breaking changes: {}\n  Non-breaking changes: {}",
            if self.is_compatible { "Compatible" } else { "Incompatible" },
            self.compatibility_score,
            self.migration_difficulty,
            self.breaking_changes.len(),
            self.non_breaking_changes.len()
        )
    }
}

impl Default for CompatibilityAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

/// Analyzes schema changes between two versions
pub struct CompatibilityAnalyzer;

impl CompatibilityAnalyzer {
    /// Analyze compatibility between old and new schemas
    pub fn analyze(
        old_properties: &[StoragePropertyDef],
        new_properties: &[StoragePropertyDef],
        schema_changes: &[SchemaChange],
    ) -> CompatibilityAnalysis {
        let mut analysis = CompatibilityAnalysis::new();

        // Build maps for quick lookup
        let old_props_map: std::collections::HashMap<_, _> =
            old_properties.iter().map(|p| (p.name.as_str(), p)).collect();
        let new_props_map: std::collections::HashMap<_, _> =
            new_properties.iter().map(|p| (p.name.as_str(), p)).collect();

        let old_prop_names: HashSet<_> = old_props_map.keys().copied().collect();
        let new_prop_names: HashSet<_> = new_props_map.keys().copied().collect();

        // Analyze each change from the change log
        for change in schema_changes {
            match &change.details {
                ChangeDetails::PropertyAdded {
                    name,
                    nullable,
                    default_value,
                    ..
                } => {
                    if *nullable {
                        analysis.add_non_breaking_change(NonBreakingChange {
                            change_type: NonBreakingChangeType::OptionalPropertyAdded,
                            property_name: Some(name.clone()),
                            description: format!("Added optional property '{}'", name),
                        });
                    } else if default_value.is_some() {
                        analysis.add_non_breaking_change(NonBreakingChange {
                            change_type: NonBreakingChangeType::DefaultedPropertyAdded,
                            property_name: Some(name.clone()),
                            description: format!(
                                "Added property '{}' with default value",
                                name
                            ),
                        });
                    } else {
                        // Non-nullable property without default - this is technically breaking
                        // because old data cannot be auto-filled
                        analysis.add_breaking_change(BreakingChange {
                            change_type: BreakingChangeType::PropertyRemoved,
                            property_name: Some(name.clone()),
                            description: format!(
                                "Added required property '{}' without default (old data incompatible)",
                                name
                            ),
                        });
                    }
                }
                ChangeDetails::PropertyRemoved { name, .. } => {
                    analysis.add_breaking_change(BreakingChange {
                        change_type: BreakingChangeType::PropertyRemoved,
                        property_name: Some(name.clone()),
                        description: format!("Removed property '{}'", name),
                    });
                }
                ChangeDetails::PropertyRenamed {
                    old_name,
                    new_name,
                } => {
                    analysis.add_non_breaking_change(NonBreakingChange {
                        change_type: NonBreakingChangeType::PropertyRenamed,
                        property_name: Some(format!("{} -> {}", old_name, new_name)),
                        description: format!("Renamed property '{}' to '{}'", old_name, new_name),
                    });
                }
                ChangeDetails::PropertyTypeModified {
                    name,
                    old_type,
                    new_type,
                } => {
                    // Check if type change is compatible
                    if Self::are_types_compatible(old_type, new_type) {
                        analysis.add_non_breaking_change(NonBreakingChange {
                            change_type: NonBreakingChangeType::PropertyRenamed,
                            property_name: Some(name.clone()),
                            description: format!(
                                "Changed property '{}' type from {:?} to {:?}",
                                name, old_type, new_type
                            ),
                        });
                    } else {
                        analysis.add_breaking_change(BreakingChange {
                            change_type: BreakingChangeType::PropertyTypeChanged,
                            property_name: Some(name.clone()),
                            description: format!(
                                "Changed property '{}' type from {:?} to {:?}",
                                name, old_type, new_type
                            ),
                        });
                    }
                }
                ChangeDetails::PropertyNullabilityChanged {
                    name,
                    was_nullable,
                    now_nullable,
                } => {
                    if !was_nullable && *now_nullable {
                        // Making more permissive (non-breaking)
                        analysis.add_non_breaking_change(NonBreakingChange {
                            change_type: NonBreakingChangeType::PropertyBecameNullable,
                            property_name: Some(name.clone()),
                            description: format!("Property '{}' became nullable", name),
                        });
                    } else {
                        // Making more restrictive (breaking)
                        analysis.add_breaking_change(BreakingChange {
                            change_type: BreakingChangeType::PropertyTypeChanged,
                            property_name: Some(name.clone()),
                            description: format!("Property '{}' became non-nullable", name),
                        });
                    }
                }
                ChangeDetails::PropertyDefaultValueChanged {
                    name,
                    old_default: _,
                    new_default: _,
                } => {
                    analysis.add_non_breaking_change(NonBreakingChange {
                        change_type: NonBreakingChangeType::DefaultedPropertyAdded,
                        property_name: Some(name.clone()),
                        description: format!("Changed default value for property '{}'", name),
                    });
                }
                ChangeDetails::PrimaryKeyChanged {
                    old_property,
                    new_property,
                } => {
                    analysis.add_breaking_change(BreakingChange {
                        change_type: BreakingChangeType::PrimaryKeyRemoved,
                        property_name: Some(format!("{} -> {}", old_property, new_property)),
                        description: format!(
                            "Changed primary key from '{}' to '{}'",
                            old_property, new_property
                        ),
                    });
                }
            }
        }

        analysis
    }

    /// Check if a type change is compatible (safe to upgrade)
    fn are_types_compatible(old_type: &crate::core::DataType, new_type: &crate::core::DataType) -> bool {
        use crate::core::DataType as DType;

        // Type upgrades that are safe:
        // - Int -> BigInt (widening)
        // - Int -> Float/Double (numeric to float)
        // - Float -> Double (widening)
        // - String conversions are generally unsafe

        match (old_type, new_type) {
            (DType::Int, DType::BigInt)
                | (DType::Int, DType::Float)
                | (DType::Int, DType::Double)
                | (DType::Float, DType::Double)
                | (DType::BigInt, DType::Double) => true,
            _ => false,
        }
    }

    /// Analyze backward compatibility
    pub fn analyze_backward_compatible(
        old_schema: &[StoragePropertyDef],
        new_schema: &[StoragePropertyDef],
    ) -> bool {
        let old_map: std::collections::HashMap<_, _> =
            old_schema.iter().map(|p| (p.name.as_str(), p)).collect();
        let new_map: std::collections::HashMap<_, _> =
            new_schema.iter().map(|p| (p.name.as_str(), p)).collect();

        // All old properties must exist in new schema
        for (name, old_prop) in &old_map {
            match new_map.get(name) {
                None => return false, // Property removed
                Some(new_prop) => {
                    // Type must be compatible
                    if old_prop.data_type != new_prop.data_type {
                        if !Self::are_types_compatible(&old_prop.data_type, &new_prop.data_type) {
                            return false;
                        }
                    }
                    // Can't become non-nullable if it was nullable
                    if old_prop.nullable && !new_prop.nullable {
                        return false;
                    }
                }
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;

    #[test]
    fn test_compatibility_analysis_creation() {
        let analysis = CompatibilityAnalysis::new();
        assert!(analysis.is_compatible);
        assert_eq!(analysis.compatibility_score, 100);
        assert_eq!(analysis.migration_difficulty, 0);
    }

    #[test]
    fn test_breaking_change_detection() {
        let mut analysis = CompatibilityAnalysis::new();
        analysis.add_breaking_change(BreakingChange {
            change_type: BreakingChangeType::PropertyRemoved,
            property_name: Some("old_field".to_string()),
            description: "Removed property 'old_field'".to_string(),
        });

        assert!(!analysis.is_compatible);
        assert!(analysis.compatibility_score < 100);
    }

    #[test]
    fn test_compatibility_analyzer() {
        let old_props = vec![StoragePropertyDef {
            name: "id".to_string(),
            data_type: DataType::String,
            nullable: false,
            default_value: None,
        }];

        let new_props = vec![
            StoragePropertyDef {
                name: "id".to_string(),
                data_type: DataType::String,
                nullable: false,
                default_value: None,
            },
            StoragePropertyDef {
                name: "email".to_string(),
                data_type: DataType::String,
                nullable: true,
                default_value: None,
            },
        ];

        let result = CompatibilityAnalyzer::analyze_backward_compatible(&old_props, &new_props);
        assert!(result); // Adding optional property is backward compatible
    }

    #[test]
    fn test_type_compatibility() {
        assert!(CompatibilityAnalyzer::are_types_compatible(
            &DataType::Int,
            &DataType::BigInt
        ));
        assert!(!CompatibilityAnalyzer::are_types_compatible(
            &DataType::BigInt,
            &DataType::Int
        ));
    }
}
