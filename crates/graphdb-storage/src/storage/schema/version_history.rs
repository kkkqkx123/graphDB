//! Version history tracking for schema migrations
//!
//! Maintains a complete history of schema versions, including snapshots
//! and change logs for each label.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::StorageResult;
use crate::storage::types::StoragePropertyDef;
use super::change::{ChangeDetails, ChangeLog, SchemaChange, SchemaObjectType};

/// Snapshot of a schema at a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaSnapshot {
    /// Version number
    pub version: u64,
    /// Timestamp in milliseconds
    pub timestamp_ms: u64,
    /// Serialized schema properties (compressed with zstd)
    pub properties_snapshot: Vec<u8>,
    /// Metadata about the snapshot
    pub metadata: HashMap<String, String>,
}

/// Version history for a single label (vertex or edge type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelVersionHistory {
    /// Label ID
    pub label_id: u32,
    /// Label name
    pub label_name: String,
    /// Object type (vertex or edge)
    pub object_type: SchemaObjectType,
    /// All versions with their snapshots
    pub versions: HashMap<u64, SchemaSnapshot>,
    /// Ordered list of version numbers
    pub version_sequence: Vec<u64>,
    /// Change log for this label
    pub change_log: ChangeLog,
    /// Compatibility matrix: for each version, which older versions are compatible
    pub compatible_versions: HashMap<u64, Vec<u64>>,
}

impl LabelVersionHistory {
    /// Create a new version history
    pub fn new(label_id: u32, label_name: String, object_type: SchemaObjectType) -> Self {
        Self {
            label_id,
            label_name: label_name.clone(),
            object_type,
            versions: HashMap::new(),
            version_sequence: Vec::new(),
            change_log: ChangeLog::new(object_type, label_id, label_name),
            compatible_versions: HashMap::new(),
        }
    }

    /// Add a snapshot for a version
    pub fn add_snapshot(
        &mut self,
        version: u64,
        timestamp_ms: u64,
        properties: &[StoragePropertyDef],
    ) -> StorageResult<()> {
        // Serialize properties to JSON, then compress with zstd
        let json_str = serde_json::to_string(properties)
            .map_err(|e| crate::core::StorageError::serialize_error(e.to_string()))?;

        let compressed = zstd::encode_all(json_str.as_bytes(), 3)
            .map_err(|e| crate::core::StorageError::compress_error(e.to_string()))?;

        let snapshot = SchemaSnapshot {
            version,
            timestamp_ms,
            properties_snapshot: compressed,
            metadata: HashMap::new(),
        };

        self.versions.insert(version, snapshot);
        if !self.version_sequence.contains(&version) {
            self.version_sequence.push(version);
            self.version_sequence.sort_unstable();
        }

        Ok(())
    }

    /// Get a snapshot for a version
    pub fn get_snapshot(&self, version: u64) -> Option<&SchemaSnapshot> {
        self.versions.get(&version)
    }

    /// Get properties from a snapshot
    pub fn get_snapshot_properties(
        &self,
        version: u64,
    ) -> StorageResult<Vec<StoragePropertyDef>> {
        let snapshot = self
            .get_snapshot(version)
            .ok_or_else(|| crate::core::StorageError::not_found(
                format!("Version {} not found for label '{}'", version, self.label_name),
            ))?;

        // Decompress and deserialize
        let decompressed = zstd::decode_all(snapshot.properties_snapshot.as_slice())
            .map_err(|e| crate::core::StorageError::decompress_error(e.to_string()))?;

        let json_str = String::from_utf8(decompressed)
            .map_err(|e| crate::core::StorageError::deserialize_error(e.to_string()))?;

        serde_json::from_str(&json_str)
            .map_err(|e| crate::core::StorageError::deserialize_error(e.to_string()))
    }

    /// Add a change to the history
    pub fn add_change(&mut self, change: SchemaChange) {
        self.change_log.add_change(change);
    }

    /// Get the latest version
    pub fn latest_version(&self) -> u64 {
        *self.version_sequence.last().unwrap_or(&1)
    }

    /// Mark versions as compatible
    pub fn mark_compatible(&mut self, from_version: u64, to_version: u64) {
        self.compatible_versions
            .entry(to_version)
            .or_insert_with(Vec::new)
            .push(from_version);
    }

    /// Check if a migration path exists (versions are compatible)
    pub fn can_migrate(&self, from_version: u64, to_version: u64) -> bool {
        if from_version >= to_version {
            return true; // No forward migration needed
        }

        // Simple path checking: allow upgrade if no breaking changes
        if let Some(compatible) = self.compatible_versions.get(&to_version) {
            if compatible.contains(&from_version) {
                return true;
            }
        }

        // Check if there are breaking changes between versions
        let breaking_between = self
            .change_log
            .changes
            .iter()
            .filter(|(v, _)| **v > from_version && **v <= to_version)
            .any(|(_, changes)| changes.iter().any(|c| c.is_breaking()));

        !breaking_between
    }

    /// Get all versions in order
    pub fn get_versions(&self) -> &[u64] {
        &self.version_sequence
    }

    /// Get breaking changes between two versions
    pub fn get_breaking_changes(&self, from_version: u64, to_version: u64) -> Vec<SchemaChange> {
        self.change_log
            .changes
            .iter()
            .filter(|(v, _)| **v > from_version && **v <= to_version)
            .flat_map(|(_, changes)| {
                changes
                    .iter()
                    .filter(|c| c.is_breaking())
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}

/// Complete schema version history for all labels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaVersionHistory {
    /// Vertex label histories
    pub vertex_histories: HashMap<u32, LabelVersionHistory>,
    /// Edge label histories
    pub edge_histories: HashMap<u32, LabelVersionHistory>,
}

impl SchemaVersionHistory {
    /// Create a new schema history
    pub fn new() -> Self {
        Self {
            vertex_histories: HashMap::new(),
            edge_histories: HashMap::new(),
        }
    }

    /// Add or update a vertex label history
    pub fn add_vertex_history(&mut self, history: LabelVersionHistory) {
        self.vertex_histories.insert(history.label_id, history);
    }

    /// Add or update an edge label history
    pub fn add_edge_history(&mut self, history: LabelVersionHistory) {
        self.edge_histories.insert(history.label_id, history);
    }

    /// Get vertex history
    pub fn get_vertex_history(&self, label_id: u32) -> Option<&LabelVersionHistory> {
        self.vertex_histories.get(&label_id)
    }

    /// Get vertex history (mutable)
    pub fn get_vertex_history_mut(&mut self, label_id: u32) -> Option<&mut LabelVersionHistory> {
        self.vertex_histories.get_mut(&label_id)
    }

    /// Get edge history
    pub fn get_edge_history(&self, label_id: u32) -> Option<&LabelVersionHistory> {
        self.edge_histories.get(&label_id)
    }

    /// Get edge history (mutable)
    pub fn get_edge_history_mut(&mut self, label_id: u32) -> Option<&mut LabelVersionHistory> {
        self.edge_histories.get_mut(&label_id)
    }

    /// Get or create vertex history
    pub fn get_or_create_vertex_history(
        &mut self,
        label_id: u32,
        label_name: String,
    ) -> &mut LabelVersionHistory {
        self.vertex_histories
            .entry(label_id)
            .or_insert_with(|| {
                LabelVersionHistory::new(label_id, label_name, SchemaObjectType::Vertex)
            })
    }

    /// Get or create edge history
    pub fn get_or_create_edge_history(
        &mut self,
        label_id: u32,
        label_name: String,
    ) -> &mut LabelVersionHistory {
        self.edge_histories
            .entry(label_id)
            .or_insert_with(|| {
                LabelVersionHistory::new(label_id, label_name, SchemaObjectType::Edge)
            })
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> StorageResult<String> {
        serde_json::to_string(self)
            .map_err(|e| crate::core::StorageError::serialize_error(e.to_string()))
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> StorageResult<Self> {
        serde_json::from_str(json)
            .map_err(|e| crate::core::StorageError::deserialize_error(e.to_string()))
    }
}

impl Default for SchemaVersionHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::DataType;

    #[test]
    fn test_label_version_history_creation() {
        let history =
            LabelVersionHistory::new(1, "User".to_string(), SchemaObjectType::Vertex);
        assert_eq!(history.label_id, 1);
        assert_eq!(history.label_name, "User");
        assert_eq!(history.latest_version(), 1);
    }

    #[test]
    fn test_snapshot_operations() {
        let mut history =
            LabelVersionHistory::new(1, "User".to_string(), SchemaObjectType::Vertex);
        let properties = vec![StoragePropertyDef::new("name".to_string(), DataType::String)];

        let result = history.add_snapshot(1, 0, &properties);
        assert!(result.is_ok());

        let retrieved = history.get_snapshot_properties(1);
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().len(), 1);
    }

    #[test]
    fn test_schema_version_history() {
        let mut schema_history = SchemaVersionHistory::new();
        let vertex_history =
            LabelVersionHistory::new(1, "User".to_string(), SchemaObjectType::Vertex);

        schema_history.add_vertex_history(vertex_history);
        assert!(schema_history.get_vertex_history(1).is_some());
    }

    #[test]
    fn test_compatibility_tracking() {
        let mut history =
            LabelVersionHistory::new(1, "User".to_string(), SchemaObjectType::Vertex);
        history.mark_compatible(1, 2);

        assert!(history.can_migrate(1, 2));
    }
}
