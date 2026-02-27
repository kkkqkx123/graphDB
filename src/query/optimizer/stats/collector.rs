//! 统计信息收集器模块
//!
//! 从存储引擎收集统计信息

use std::sync::Arc;

use crate::core::{StorageError, Vertex};
use crate::core::types::EdgeDirection;
use crate::storage::StorageClient;

use super::{TagStatistics, EdgeTypeStatistics, PropertyStatistics};

/// 统计信息收集器
pub struct StatisticsCollector<S: StorageClient> {
    storage: Arc<S>,
}

/// 统计信息集合
#[derive(Debug, Clone, Default)]
pub struct StatisticsCollection {
    /// 标签统计信息列表
    pub tag_stats: Vec<TagStatistics>,
    /// 边类型统计信息列表
    pub edge_stats: Vec<EdgeTypeStatistics>,
    /// 属性统计信息列表
    pub property_stats: Vec<PropertyStatistics>,
}

impl StatisticsCollection {
    /// 创建新的统计信息集合
    pub fn new() -> Self {
        Self {
            tag_stats: Vec::new(),
            edge_stats: Vec::new(),
            property_stats: Vec::new(),
        }
    }
}

impl<S: StorageClient> StatisticsCollector<S> {
    /// 创建新的统计信息收集器
    pub fn new(storage: Arc<S>) -> Self {
        Self { storage }
    }

    /// 收集标签统计信息
    pub fn collect_tag_stats(&self, space: &str, tag_name: &str) -> Result<TagStatistics, StorageError> {
        let mut stats = TagStatistics::new(tag_name.to_string());

        // 扫描该标签的所有顶点
        let vertices = self.storage.scan_vertices_by_tag(space, tag_name)?;
        stats.vertex_count = vertices.len() as u64;

        if stats.vertex_count > 0 {
            // 计算平均顶点大小
            let total_size: usize = vertices.iter()
                .map(|v| v.estimated_size())
                .sum();
            stats.avg_vertex_size = total_size / vertices.len();

            // 计算平均度数
            let (avg_out, avg_in) = self.calculate_average_degrees(space, &vertices)?;
            stats.avg_out_degree = avg_out;
            stats.avg_in_degree = avg_in;
        }

        Ok(stats)
    }

    /// 计算顶点的平均出度和入度
    fn calculate_average_degrees(
        &self,
        space: &str,
        vertices: &[Vertex],
    ) -> Result<(f64, f64), StorageError> {
        let mut total_out_degree: usize = 0;
        let mut total_in_degree: usize = 0;

        for vertex in vertices {
            // 获取出边
            let out_edges = self.storage.get_node_edges(space, vertex.vid(), EdgeDirection::Out)?;
            total_out_degree += out_edges.len();

            // 获取入边
            let in_edges = self.storage.get_node_edges(space, vertex.vid(), EdgeDirection::In)?;
            total_in_degree += in_edges.len();
        }

        let count = vertices.len();
        let avg_out = if count > 0 {
            total_out_degree as f64 / count as f64
        } else {
            0.0
        };
        let avg_in = if count > 0 {
            total_in_degree as f64 / count as f64
        } else {
            0.0
        };

        Ok((avg_out, avg_in))
    }

    /// 收集边类型统计信息
    pub fn collect_edge_stats(&self, space: &str, edge_type: &str) -> Result<EdgeTypeStatistics, StorageError> {
        let mut stats = EdgeTypeStatistics::new(edge_type.to_string());

        // 扫描该类型的所有边
        let edges = self.storage.scan_edges_by_type(space, edge_type)?;
        stats.edge_count = edges.len() as u64;

        if stats.edge_count > 0 {
            // 计算唯一源顶点和目标顶点数
            let mut unique_src = std::collections::HashSet::new();
            let mut unique_dst = std::collections::HashSet::new();

            for edge in &edges {
                unique_src.insert(edge.src().hash_value());
                unique_dst.insert(edge.dst().hash_value());
            }

            stats.unique_src_vertices = unique_src.len() as u64;
            stats.unique_dst_vertices = unique_dst.len() as u64;

            // 计算平均出度和入度
            stats.avg_out_degree = if stats.unique_src_vertices > 0 {
                stats.edge_count as f64 / stats.unique_src_vertices as f64
            } else {
                0.0
            };
            stats.avg_in_degree = if stats.unique_dst_vertices > 0 {
                stats.edge_count as f64 / stats.unique_dst_vertices as f64
            } else {
                0.0
            };
        }

        Ok(stats)
    }

    /// 收集所有统计信息
    pub fn collect_all_stats(&self, space: &str) -> Result<StatisticsCollection, StorageError> {
        let mut collection = StatisticsCollection::new();

        // 获取所有标签
        let tags = self.storage.list_tags(space)?;
        for tag_info in tags {
            let stats = self.collect_tag_stats(space, &tag_info.tag_name)?;
            collection.tag_stats.push(stats);
        }

        // 获取所有边类型
        let edge_types = self.storage.list_edge_types(space)?;
        for edge_type_info in edge_types {
            let stats = self.collect_edge_stats(space, &edge_type_info.edge_type_name)?;
            collection.edge_stats.push(stats);
        }

        Ok(collection)
    }
}
