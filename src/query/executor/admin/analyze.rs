//! AnalyzeExecutor - 分析执行器
//!
//! 负责收集和更新数据库统计信息，用于查询优化。

use std::sync::Arc;

use parking_lot::Mutex;

use crate::core::{DataSet, Value};
use crate::core::types::EdgeDirection;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, HasStorage};
use crate::query::optimizer::stats::{StatisticsManager, TagStatistics, EdgeTypeStatistics};
use crate::storage::StorageClient;

/// 分析目标类型
#[derive(Debug, Clone)]
pub enum AnalyzeTarget {
    /// 分析所有对象
    All,
    /// 分析指定标签
    Tag(String),
    /// 分析指定边类型
    EdgeType(String),
    /// 分析指定属性
    Property { tag: Option<String>, property: String },
}

/// 分析执行器
///
/// 该执行器负责收集数据库统计信息，用于查询优化器的代价计算。
/// 通过 ANALYZE 命令触发执行。
#[derive(Debug)]
pub struct AnalyzeExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    target: AnalyzeTarget,
    stats_manager: Arc<Mutex<StatisticsManager>>,
}

impl<S: StorageClient> AnalyzeExecutor<S> {
    /// 创建新的 AnalyzeExecutor
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            base: BaseExecutor::new(id, "AnalyzeExecutor".to_string(), storage),
            target: AnalyzeTarget::All,
            stats_manager: Arc::new(Mutex::new(StatisticsManager::new())),
        }
    }

    /// 创建带目标的 AnalyzeExecutor
    pub fn with_target(
        id: i64,
        storage: Arc<Mutex<S>>,
        target: AnalyzeTarget,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "AnalyzeExecutor".to_string(), storage),
            target,
            stats_manager: Arc::new(Mutex::new(StatisticsManager::new())),
        }
    }

    /// 设置分析目标
    pub fn set_target(&mut self, target: AnalyzeTarget) {
        self.target = target;
    }

    /// 获取统计信息管理器
    pub fn stats_manager(&self) -> Arc<Mutex<StatisticsManager>> {
        self.stats_manager.clone()
    }

    /// 收集标签统计信息
    fn collect_tag_stats(
        &self,
        storage: &S,
        space: &str,
        tag_name: &str,
    ) -> Result<TagStatistics, crate::core::StorageError> {
        let mut stats = TagStatistics::new(tag_name.to_string());

        // 扫描该标签的所有顶点
        let vertices = storage.scan_vertices_by_tag(space, tag_name)?;
        stats.vertex_count = vertices.len() as u64;

        if stats.vertex_count > 0 {
            // 计算平均顶点大小
            let total_size: usize = vertices.iter()
                .map(|v| v.estimated_size())
                .sum();
            stats.avg_vertex_size = total_size / vertices.len();

            // 计算平均度数
            let (avg_out, avg_in) = self.calculate_average_degrees(storage, space, &vertices)?;
            stats.avg_out_degree = avg_out;
            stats.avg_in_degree = avg_in;
        }

        Ok(stats)
    }

    /// 计算顶点的平均出度和入度
    fn calculate_average_degrees(
        &self,
        storage: &S,
        space: &str,
        vertices: &[crate::core::Vertex],
    ) -> Result<(f64, f64), crate::core::StorageError> {
        let mut total_out_degree: usize = 0;
        let mut total_in_degree: usize = 0;

        for vertex in vertices {
            // 获取出边
            let out_edges = storage.get_node_edges(space, vertex.vid(), EdgeDirection::Out)?;
            total_out_degree += out_edges.len();

            // 获取入边
            let in_edges = storage.get_node_edges(space, vertex.vid(), EdgeDirection::In)?;
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
    fn collect_edge_stats(
        &self,
        storage: &S,
        space: &str,
        edge_type: &str,
    ) -> Result<EdgeTypeStatistics, crate::core::StorageError> {
        let mut stats = EdgeTypeStatistics::new(edge_type.to_string());

        // 扫描该类型的所有边
        let edges = storage.scan_edges_by_type(space, edge_type)?;
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

    /// 执行分析并返回结果数据集
    fn execute_analysis(&self, space: &str) -> crate::query::executor::base::DBResult<DataSet> {
        let storage = self.get_storage();
        let storage_guard = storage.lock();

        let mut rows = Vec::new();

        match &self.target {
            AnalyzeTarget::All => {
                // 获取所有标签
                let tags = storage_guard.list_tags(space)?;
                for tag_info in &tags {
                    let stats = self.collect_tag_stats(&*storage_guard, space, &tag_info.tag_name)?;
                    
                    // 更新统计信息管理器
                    {
                        let manager = self.stats_manager.lock();
                        manager.update_tag_stats(stats.clone());
                    }

                    rows.push(vec![
                        Value::String("TAG".to_string()),
                        Value::String(stats.tag_name.clone()),
                        Value::Int(stats.vertex_count as i64),
                        Value::Float(stats.avg_out_degree),
                        Value::Float(stats.avg_in_degree),
                    ]);
                }

                // 获取所有边类型
                let edge_types = storage_guard.list_edge_types(space)?;
                for edge_type_info in &edge_types {
                    let stats = self.collect_edge_stats(&*storage_guard, space, &edge_type_info.edge_type_name)?;

                    // 更新统计信息管理器
                    {
                        let manager = self.stats_manager.lock();
                        manager.update_edge_stats(stats.clone());
                    }

                    rows.push(vec![
                        Value::String("EDGE".to_string()),
                        Value::String(stats.edge_type.clone()),
                        Value::Int(stats.edge_count as i64),
                        Value::Float(stats.avg_out_degree),
                        Value::Float(stats.avg_in_degree),
                    ]);
                }
            }
            AnalyzeTarget::Tag(tag_name) => {
                let stats = self.collect_tag_stats(&*storage_guard, space, tag_name)?;

                // 更新统计信息管理器
                {
                    let manager = self.stats_manager.lock();
                    manager.update_tag_stats(stats.clone());
                }

                rows.push(vec![
                    Value::String("TAG".to_string()),
                    Value::String(stats.tag_name.clone()),
                    Value::Int(stats.vertex_count as i64),
                    Value::Float(stats.avg_out_degree),
                    Value::Float(stats.avg_in_degree),
                ]);
            }
            AnalyzeTarget::EdgeType(edge_type) => {
                let stats = self.collect_edge_stats(&*storage_guard, space, edge_type)?;

                // 更新统计信息管理器
                {
                    let manager = self.stats_manager.lock();
                    manager.update_edge_stats(stats.clone());
                }

                rows.push(vec![
                    Value::String("EDGE".to_string()),
                    Value::String(stats.edge_type.clone()),
                    Value::Int(stats.edge_count as i64),
                    Value::Float(stats.avg_out_degree),
                    Value::Float(stats.avg_in_degree),
                ]);
            }
            AnalyzeTarget::Property { tag, property } => {
                // 属性统计信息收集
                // 目前简化实现，返回基本信息
                rows.push(vec![
                    Value::String("PROPERTY".to_string()),
                    Value::String(format!(
                        "{}.{}",
                        tag.as_deref().unwrap_or("*"),
                        property
                    )),
                    Value::Int(0),
                    Value::Float(0.0),
                    Value::Float(0.0),
                ]);
            }
        }

        Ok(DataSet {
            col_names: vec![
                "Type".to_string(),
                "Name".to_string(),
                "Count".to_string(),
                "Avg Out Degree".to_string(),
                "Avg In Degree".to_string(),
            ],
            rows,
        })
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for AnalyzeExecutor<S> {
    fn execute(&mut self) -> crate::query::executor::base::DBResult<ExecutionResult> {
        // 获取当前空间名称，从上下文变量中获取
        let space = self
            .base
            .context
            .get_variable("current_space")
            .and_then(|v| match v {
                Value::String(s) => Some(s.clone()),
                _ => None,
            })
            .unwrap_or_else(|| "default".to_string());

        match self.execute_analysis(&space) {
            Ok(dataset) => Ok(ExecutionResult::DataSet(dataset)),
            Err(e) => Ok(ExecutionResult::Error(format!("ANALYZE failed: {}", e))),
        }
    }

    fn open(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.open()
    }

    fn close(&mut self) -> crate::query::executor::base::DBResult<()> {
        self.base.close()
    }

    fn is_open(&self) -> bool {
        self.base.is_open()
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "AnalyzeExecutor"
    }

    fn description(&self) -> &str {
        "Collects database statistics for query optimization"
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for AnalyzeExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_target_clone() {
        let target = AnalyzeTarget::Tag("Person".to_string());
        let cloned = target.clone();

        match cloned {
            AnalyzeTarget::Tag(name) => assert_eq!(name, "Person"),
            _ => panic!("Expected Tag target"),
        }
    }

    #[test]
    fn test_analyze_target_debug() {
        let target = AnalyzeTarget::All;
        let debug_str = format!("{:?}", target);
        assert!(debug_str.contains("All"));
    }
}
