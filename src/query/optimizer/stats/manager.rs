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
    /// 标签统计信息（以标签名称为键）
    tag_stats: Arc<RwLock<HashMap<String, TagStatistics>>>,
    /// 标签ID到标签名称的映射
    tag_id_to_name: Arc<RwLock<HashMap<i32, String>>>,
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
            tag_id_to_name: Arc::new(RwLock::new(HashMap::new())),
            edge_stats: Arc::new(RwLock::new(HashMap::new())),
            property_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册标签ID到名称的映射
    pub fn register_tag_id(&self, tag_id: i32, tag_name: String) {
        self.tag_id_to_name.write().insert(tag_id, tag_name);
    }

    /// 根据标签ID获取标签名称
    pub fn get_tag_name_by_id(&self, tag_id: i32) -> Option<String> {
        self.tag_id_to_name.read().get(&tag_id).cloned()
    }

    /// 根据标签ID获取标签统计信息
    pub fn get_tag_stats_by_id(&self, tag_id: i32) -> Option<TagStatistics> {
        let tag_name = self.get_tag_name_by_id(tag_id)?;
        self.get_tag_stats(&tag_name)
    }

    /// 根据标签ID获取顶点数量
    pub fn get_vertex_count_by_id(&self, tag_id: i32) -> u64 {
        self.get_tag_stats_by_id(tag_id)
            .map(|s| s.vertex_count)
            .unwrap_or(0)
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
        self.tag_id_to_name.write().clear();
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
            tag_id_to_name: Arc::new(RwLock::new(self.tag_id_to_name.read().clone())),
            edge_stats: Arc::new(RwLock::new(self.edge_stats.read().clone())),
            property_stats: Arc::new(RwLock::new(self.property_stats.read().clone())),
        }
    }
}
