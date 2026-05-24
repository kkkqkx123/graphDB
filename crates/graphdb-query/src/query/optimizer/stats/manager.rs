//! Statistical Information Manager Module
//!
//! Centralized management of all statistical information, with thread-safe access.

use dashmap::DashMap;
use std::sync::Arc;

use super::{EdgeTypeStatistics, PropertyCombinationStats, PropertyStatistics, TagStatistics};

/// Statistical Information Manager
///
/// Centralized management of all statistical information, ensuring thread-safe access.
#[derive(Debug)]
pub struct StatisticsManager {
    /// Tag statistics information (with tag names as keys)
    tag_stats: Arc<DashMap<String, TagStatistics>>,
    /// Mapping from Tag ID to Tag Name
    tag_id_to_name: Arc<DashMap<i32, String>>,
    /// Type statistics information for edges
    edge_stats: Arc<DashMap<String, EdgeTypeStatistics>>,
    /// Attribute statistics information
    property_stats: Arc<DashMap<String, PropertyStatistics>>,
    /// Property combination statistics for GROUP BY cardinality estimation
    property_combo_stats: Arc<DashMap<String, PropertyCombinationStats>>,
}

impl StatisticsManager {
    /// Create a new statistical information manager.
    pub fn new() -> Self {
        Self {
            tag_stats: Arc::new(DashMap::new()),
            tag_id_to_name: Arc::new(DashMap::new()),
            edge_stats: Arc::new(DashMap::new()),
            property_stats: Arc::new(DashMap::new()),
            property_combo_stats: Arc::new(DashMap::new()),
        }
    }

    /// Mapping of registered tag IDs to their corresponding names
    pub fn register_tag_id(&self, tag_id: i32, tag_name: String) {
        self.tag_id_to_name.insert(tag_id, tag_name);
    }

    /// Retrieve the tag name based on the tag ID.
    pub fn get_tag_name_by_id(&self, tag_id: i32) -> Option<String> {
        self.tag_id_to_name.get(&tag_id).map(|v| v.clone())
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
        self.tag_stats.get(tag_name).map(|v| v.clone())
    }

    /// Update the tag statistics information.
    pub fn update_tag_stats(&self, stats: TagStatistics) {
        self.tag_stats.insert(stats.tag_name.clone(), stats);
    }

    /// Obtain the number of vertices
    pub fn get_vertex_count(&self, tag_name: &str) -> u64 {
        self.get_tag_stats(tag_name)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
    }

    /// Obtain statistical information about the types of edges.
    pub fn get_edge_stats(&self, edge_type: &str) -> Option<EdgeTypeStatistics> {
        self.edge_stats.get(edge_type).map(|v| v.clone())
    }

    /// Update the statistics information on edge types.
    pub fn update_edge_stats(&self, stats: EdgeTypeStatistics) {
        self.edge_stats.insert(stats.edge_type.clone(), stats);
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
        self.property_stats.get(&key).map(|v| v.clone())
    }

    /// Update attribute statistics information
    pub fn update_property_stats(&self, stats: PropertyStatistics) {
        let key = match &stats.tag_name {
            Some(tag) => format!("{}.{}", tag, stats.property_name),
            None => stats.property_name.clone(),
        };
        self.property_stats.insert(key, stats);
    }

    /// Clear all statistical information.
    pub fn clear_all(&self) {
        self.tag_stats.clear();
        self.tag_id_to_name.clear();
        self.edge_stats.clear();
        self.property_stats.clear();
        self.property_combo_stats.clear();
    }

    /// Get property combination statistics for GROUP BY cardinality estimation.
    pub fn get_property_combo_stats(
        &self,
        tag_name: &str,
        properties: &[String],
    ) -> Option<PropertyCombinationStats> {
        let key = format!("{}.{}", tag_name, properties.join("."));
        self.property_combo_stats.get(&key).map(|v| v.clone())
    }

    /// Update property combination statistics.
    pub fn update_property_combo_stats(&self, stats: PropertyCombinationStats) {
        self.property_combo_stats.insert(stats.key.clone(), stats);
    }

    /// Get combined cardinality for a set of properties.
    /// Returns None if no statistics are available.
    pub fn get_combined_cardinality(
        &self,
        tag_name: Option<&str>,
        properties: &[String],
    ) -> Option<u64> {
        let tag = tag_name?;
        self.get_property_combo_stats(tag, properties)
            .map(|s| s.estimated_cardinality())
    }

    /// Retrieve all tag names
    pub fn get_all_tags(&self) -> Vec<String> {
        self.tag_stats.iter().map(|k| k.key().clone()).collect()
    }

    /// Obtain the names of all edge types.
    pub fn get_all_edge_types(&self) -> Vec<String> {
        self.edge_stats.iter().map(|k| k.key().clone()).collect()
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
            tag_stats: Arc::clone(&self.tag_stats),
            tag_id_to_name: Arc::clone(&self.tag_id_to_name),
            edge_stats: Arc::clone(&self.edge_stats),
            property_stats: Arc::clone(&self.property_stats),
            property_combo_stats: Arc::clone(&self.property_combo_stats),
        }
    }
}
