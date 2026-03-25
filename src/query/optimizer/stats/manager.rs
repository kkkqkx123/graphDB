//! Statistical Information Manager Module
//!
//! Centralized management of all statistical information, with thread-safe access.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use super::{EdgeTypeStatistics, PropertyStatistics, TagStatistics};

/// Statistical Information Manager
///
/// Centralized management of all statistical information, ensuring thread-safe access.
#[derive(Debug)]
pub struct StatisticsManager {
    /// Tag statistics information (with tag names as keys)
    tag_stats: Arc<RwLock<HashMap<String, TagStatistics>>>,
    /// Mapping from Tag ID to Tag Name
    tag_id_to_name: Arc<RwLock<HashMap<i32, String>>>,
    /// Type statistics information for edges
    edge_stats: Arc<RwLock<HashMap<String, EdgeTypeStatistics>>>,
    /// Attribute statistics information
    property_stats: Arc<RwLock<HashMap<String, PropertyStatistics>>>,
}

impl StatisticsManager {
    /// Create a new statistical information manager.
    pub fn new() -> Self {
        Self {
            tag_stats: Arc::new(RwLock::new(HashMap::new())),
            tag_id_to_name: Arc::new(RwLock::new(HashMap::new())),
            edge_stats: Arc::new(RwLock::new(HashMap::new())),
            property_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Mapping of registered tag IDs to their corresponding names
    pub fn register_tag_id(&self, tag_id: i32, tag_name: String) {
        self.tag_id_to_name.write().insert(tag_id, tag_name);
    }

    /// Retrieve the tag name based on the tag ID.
    pub fn get_tag_name_by_id(&self, tag_id: i32) -> Option<String> {
        self.tag_id_to_name.read().get(&tag_id).cloned()
    }

    /// Retrieve tag statistics based on the tag ID.
    pub fn get_tag_stats_by_id(&self, tag_id: i32) -> Option<TagStatistics> {
        let tag_name = self.get_tag_name_by_id(tag_id)?;
        self.get_tag_stats(&tag_name)
    }

    /// Get the number of vertices based on the tag ID.
    pub fn get_vertex_count_by_id(&self, tag_id: i32) -> u64 {
        self.get_tag_stats_by_id(tag_id)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
    }

    /// Obtain tag statistics information
    pub fn get_tag_stats(&self, tag_name: &str) -> Option<TagStatistics> {
        self.tag_stats.read().get(tag_name).cloned()
    }

    /// Update the tag statistics information.
    pub fn update_tag_stats(&self, stats: TagStatistics) {
        self.tag_stats.write().insert(stats.tag_name.clone(), stats);
    }

    /// Obtain the number of vertices
    pub fn get_vertex_count(&self, tag_name: &str) -> u64 {
        self.get_tag_stats(tag_name)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
    }

    /// Obtain statistical information about the types of edges.
    pub fn get_edge_stats(&self, edge_type: &str) -> Option<EdgeTypeStatistics> {
        self.edge_stats.read().get(edge_type).cloned()
    }

    /// Update the statistics information on edge types.
    pub fn update_edge_stats(&self, stats: EdgeTypeStatistics) {
        self.edge_stats
            .write()
            .insert(stats.edge_type.clone(), stats);
    }

    /// Obtain the number of edges
    pub fn get_edge_count(&self, edge_type: &str) -> u64 {
        self.get_edge_stats(edge_type)
            .map(|s| s.edge_count)
            .unwrap_or(0)
    }

    /// Obtain attribute statistics information
    pub fn get_property_stats(
        &self,
        tag_name: Option<&str>,
        property_name: &str,
    ) -> Option<PropertyStatistics> {
        let key = match tag_name {
            Some(tag) => format!("{}.{}", tag, property_name),
            None => property_name.to_string(),
        };
        self.property_stats.read().get(&key).cloned()
    }

    /// Update attribute statistics information
    pub fn update_property_stats(&self, stats: PropertyStatistics) {
        let key = match &stats.tag_name {
            Some(tag) => format!("{}.{}", tag, stats.property_name),
            None => stats.property_name.clone(),
        };
        self.property_stats.write().insert(key, stats);
    }

    /// Clear all statistical information.
    pub fn clear_all(&self) {
        self.tag_stats.write().clear();
        self.tag_id_to_name.write().clear();
        self.edge_stats.write().clear();
        self.property_stats.write().clear();
    }

    /// Retrieve all tag names
    pub fn get_all_tags(&self) -> Vec<String> {
        self.tag_stats.read().keys().cloned().collect()
    }

    /// Obtain the names of all edge types.
    pub fn get_all_edge_types(&self) -> Vec<String> {
        self.edge_stats.read().keys().cloned().collect()
    }
}

impl Default for StatisticsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StatisticsManager {
    fn clone(&self) -> Self {
        Self {
            tag_stats: Arc::new(RwLock::new(self.tag_stats.read().clone())),
            tag_id_to_name: Arc::new(RwLock::new(self.tag_id_to_name.read().clone())),
            edge_stats: Arc::new(RwLock::new(self.edge_stats.read().clone())),
            property_stats: Arc::new(RwLock::new(self.property_stats.read().clone())),
        }
    }
}
