//! 统计信息管理器模块
//!
//! 统一管理所有统计信息，提供线程安全的访问

use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

use super::{TagStatistics, EdgeTypeStatistics, PropertyStatistics};

/// 统计信息管理器
///
/// 统一管理所有统计信息，提供线程安全的访问
#[derive(Debug)]
pub struct StatisticsManager {
    /// 标签统计信息
    tag_stats: Arc<RwLock<HashMap<String, TagStatistics>>>,
    /// 边类型统计信息
    edge_stats: Arc<RwLock<HashMap<String, EdgeTypeStatistics>>>,
    /// 属性统计信息
    property_stats: Arc<RwLock<HashMap<String, PropertyStatistics>>>,
}

impl StatisticsManager {
    /// 创建新的统计信息管理器
    pub fn new() -> Self {
        Self {
            tag_stats: Arc::new(RwLock::new(HashMap::new())),
            edge_stats: Arc::new(RwLock::new(HashMap::new())),
            property_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取标签统计信息
    pub fn get_tag_stats(&self, tag_name: &str) -> Option<TagStatistics> {
        self.tag_stats.read().get(tag_name).cloned()
    }

    /// 更新标签统计信息
    pub fn update_tag_stats(&self, stats: TagStatistics) {
        self.tag_stats.write().insert(stats.tag_name.clone(), stats);
    }

    /// 获取顶点数量
    pub fn get_vertex_count(&self, tag_name: &str) -> u64 {
        self.get_tag_stats(tag_name)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
    }

    /// 获取边类型统计信息
    pub fn get_edge_stats(&self, edge_type: &str) -> Option<EdgeTypeStatistics> {
        self.edge_stats.read().get(edge_type).cloned()
    }

    /// 更新边类型统计信息
    pub fn update_edge_stats(&self, stats: EdgeTypeStatistics) {
        self.edge_stats.write().insert(stats.edge_type.clone(), stats);
    }

    /// 获取边数量
    pub fn get_edge_count(&self, edge_type: &str) -> u64 {
        self.get_edge_stats(edge_type)
            .map(|s| s.edge_count)
            .unwrap_or(0)
    }

    /// 获取属性统计信息
    pub fn get_property_stats(&self, tag_name: Option<&str>, property_name: &str) -> Option<PropertyStatistics> {
        let key = match tag_name {
            Some(tag) => format!("{}.{}", tag, property_name),
            None => property_name.to_string(),
        };
        self.property_stats.read().get(&key).cloned()
    }

    /// 更新属性统计信息
    pub fn update_property_stats(&self, stats: PropertyStatistics) {
        let key = match &stats.tag_name {
            Some(tag) => format!("{}.{}", tag, stats.property_name),
            None => stats.property_name.clone(),
        };
        self.property_stats.write().insert(key, stats);
    }

    /// 清除所有统计信息
    pub fn clear_all(&self) {
        self.tag_stats.write().clear();
        self.edge_stats.write().clear();
        self.property_stats.write().clear();
    }

    /// 获取所有标签名称
    pub fn get_all_tags(&self) -> Vec<String> {
        self.tag_stats.read().keys().cloned().collect()
    }

    /// 获取所有边类型名称
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
            edge_stats: Arc::new(RwLock::new(self.edge_stats.read().clone())),
            property_stats: Arc::new(RwLock::new(self.property_stats.read().clone())),
        }
    }
}
