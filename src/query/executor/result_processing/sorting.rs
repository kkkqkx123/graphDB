//! 排序执行器
//!
//! SortExecutor - ORDER BY 执行

use async_trait::async_trait;
use std::sync::{Arc, Mutex};

use crate::core::Value;
use crate::query::QueryError;
use crate::storage::StorageEngine;
use crate::query::executor::base::{BaseExecutor, ExecutionResult, Executor, InputExecutor};

/// 排序因子定义
#[derive(Debug, Clone)]
pub struct SortFactor {
    pub column: String,                // 排序列名
    pub ascending: bool,               // 排序方向
}

impl SortFactor {
    pub fn new(column: String, ascending: bool) -> Self {
        Self { column, ascending }
    }
}

/// SortExecutor - 排序执行器
///
/// 执行排序操作，支持多列排序和升序/降序
pub struct SortExecutor<S: StorageEngine> {
    base: BaseExecutor<S>,
    sort_factors: Vec<SortFactor>,     // 排序因子
    input_executor: Option<Box<dyn Executor<S>>>,
}

impl<S: StorageEngine> SortExecutor<S> {
    pub fn new(
        id: usize,
        storage: Arc<Mutex<S>>,
        sort_factors: Vec<SortFactor>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "SortExecutor".to_string(), storage),
            sort_factors,
            input_executor: None,
        }
    }

    /// 从行中提取排序键值
    fn extract_sort_key(&self, row: &[Value], col_names: &[String]) -> Vec<Value> {
        self.sort_factors
            .iter()
            .map(|factor| {
                // 根据列名找到对应的值
                if let Some(index) = col_names.iter().position(|name| name == &factor.column) {
                    if index < row.len() {
                        row[index].clone()
                    } else {
                        Value::Null(crate::core::value::NullType::Null)
                    }
                } else {
                    Value::Null(crate::core::value::NullType::Null)
                }
            })
            .collect()
    }

    /// 创建比较器函数
    fn create_comparator<'a>(&'a self, col_names: &'a [String]) -> impl Fn(&[Value], &[Value]) -> std::cmp::Ordering + 'a {
        move |a: &[Value], b: &[Value]| {
            let key_a = self.extract_sort_key(a, col_names);
            let key_b = self.extract_sort_key(b, col_names);

            for (i, factor) in self.sort_factors.iter().enumerate() {
                if i >= key_a.len() || i >= key_b.len() {
                    continue;
                }

                let value_a = &key_a[i];
                let value_b = &key_b[i];

                // 处理空值
                if value_a.is_null() && value_b.is_null() {
                    continue;
                }
                if value_a.is_null() {
                    return std::cmp::Ordering::Less;
                }
                if value_b.is_null() {
                    return std::cmp::Ordering::Greater;
                }

                // 比较值
                let ordering = value_a.cmp(value_b);
                if ordering != std::cmp::Ordering::Equal {
                    return if factor.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                }
            }

            std::cmp::Ordering::Equal
        }
    }

    /// 排序数据集
    fn sort_dataset(
        &self,
        mut dataset: crate::core::value::DataSet,
    ) -> crate::core::value::DataSet {
        if dataset.rows.is_empty() {
            return dataset;
        }

        let col_names = dataset.col_names.clone();
        let comparator = self.create_comparator(&col_names);
        dataset.rows.sort_by(|a, b| comparator(a, b));
        dataset
    }

    /// 排序顶点列表
    fn sort_vertices(
        &self,
        mut vertices: Vec<crate::core::Vertex>,
    ) -> Vec<crate::core::Vertex> {
        if vertices.is_empty() {
            return vertices;
        }

        // 创建比较器
        vertices.sort_by(|a, b| {
            for factor in &self.sort_factors {
                let ordering = match factor.column.as_str() {
                    "vid" => a.vid.cmp(&b.vid),
                    "tag" => {
                        // 比较标签
                        let empty = String::new();
                        let tag_a = a.tags.first().map(|t| &t.name).unwrap_or(&empty);
                        let tag_b = b.tags.first().map(|t| &t.name).unwrap_or(&empty);
                        tag_a.cmp(tag_b)
                    }
                    _ => {
                        // 比较属性值 - 简化实现，只比较第一个标签的第一个属性
                        let prop_a = a.tags.first()
                            .and_then(|tag| tag.properties.get(&factor.column))
                            .unwrap_or(&Value::Null(crate::core::value::NullType::Null));
                        let prop_b = b.tags.first()
                            .and_then(|tag| tag.properties.get(&factor.column))
                            .unwrap_or(&Value::Null(crate::core::value::NullType::Null));
                        prop_a.cmp(prop_b)
                    }
                };

                if ordering != std::cmp::Ordering::Equal {
                    return if factor.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                }
            }
            std::cmp::Ordering::Equal
        });

        vertices
    }

    /// 排序边列表
    fn sort_edges(
        &self,
        mut edges: Vec<crate::core::Edge>,
    ) -> Vec<crate::core::Edge> {
        if edges.is_empty() {
            return edges;
        }

        edges.sort_by(|a, b| {
            for factor in &self.sort_factors {
                let ordering = match factor.column.as_str() {
                    "src" => a.src.cmp(&b.src),
                    "dst" => a.dst.cmp(&b.dst),
                    "edge_type" => a.edge_type.cmp(&b.edge_type),
                    "ranking" => a.ranking.cmp(&b.ranking),
                    _ => {
                        // 比较属性值
                        let prop_a = a.props.get(&factor.column).unwrap_or(&Value::Null(crate::core::value::NullType::Null));
                        let prop_b = b.props.get(&factor.column).unwrap_or(&Value::Null(crate::core::value::NullType::Null));
                        prop_a.cmp(prop_b)
                    }
                };

                if ordering != std::cmp::Ordering::Equal {
                    return if factor.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                }
            }
            std::cmp::Ordering::Equal
        });

        edges
    }

    /// 排序值列表
    fn sort_values(&self, mut values: Vec<Value>) -> Vec<Value> {
        if values.is_empty() {
            return values;
        }

        values.sort();
        
        // 如果指定了降序，则反转结果
        if let Some(factor) = self.sort_factors.first() {
            if !factor.ascending {
                values.reverse();
            }
        }

        values
    }

    /// 排序路径列表
    fn sort_paths(&self, mut paths: Vec<crate::core::vertex_edge_path::Path>) -> Vec<crate::core::vertex_edge_path::Path> {
        if paths.is_empty() {
            return paths;
        }

        paths.sort_by(|a, b| {
            for factor in &self.sort_factors {
                let ordering = match factor.column.as_str() {
                    "length" => a.len().cmp(&b.len()),
                    "src" => a.src.vid.cmp(&b.src.vid),
                    "dst" => {
                        // 获取路径的最后一个顶点作为目标
                        let dst_a = a.steps.last().map(|s| &s.dst.vid).unwrap_or(&a.src.vid);
                        let dst_b = b.steps.last().map(|s| &s.dst.vid).unwrap_or(&b.src.vid);
                        dst_a.cmp(dst_b)
                    },
                    _ => std::cmp::Ordering::Equal,
                };

                if ordering != std::cmp::Ordering::Equal {
                    return if factor.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    };
                }
            }
            std::cmp::Ordering::Equal
        });

        paths
    }
}

impl<S: StorageEngine> InputExecutor<S> for SortExecutor<S> {
    fn set_input(&mut self, input: Box<dyn Executor<S>>) {
        self.input_executor = Some(input);
    }

    fn get_input(&self) -> Option<&Box<dyn Executor<S>>> {
        self.input_executor.as_ref()
    }
}

#[async_trait]
impl<S: StorageEngine + Send + 'static> Executor<S> for SortExecutor<S> {
    async fn execute(&mut self) -> Result<ExecutionResult, QueryError> {
        // 首先执行输入执行器（如果存在）
        let input_result = if let Some(ref mut input_exec) = self.input_executor {
            input_exec.execute().await?
        } else {
            // 如果没有输入执行器，返回空结果
            ExecutionResult::DataSet(crate::core::value::DataSet::new())
        };

        // 对结果应用排序
        let sorted_result = match input_result {
            ExecutionResult::DataSet(dataset) => {
                let sorted_dataset = self.sort_dataset(dataset);
                ExecutionResult::DataSet(sorted_dataset)
            }
            ExecutionResult::Vertices(vertices) => {
                let sorted_vertices = self.sort_vertices(vertices);
                ExecutionResult::Vertices(sorted_vertices)
            }
            ExecutionResult::Edges(edges) => {
                let sorted_edges = self.sort_edges(edges);
                ExecutionResult::Edges(sorted_edges)
            }
            ExecutionResult::Values(values) => {
                let sorted_values = self.sort_values(values);
                ExecutionResult::Values(sorted_values)
            }
            ExecutionResult::Paths(paths) => {
                let sorted_paths = self.sort_paths(paths);
                ExecutionResult::Paths(sorted_paths)
            }
            ExecutionResult::Count(count) => {
                // 计数结果不需要排序
                ExecutionResult::Count(count)
            }
            ExecutionResult::Success => ExecutionResult::Success,
        };

        Ok(sorted_result)
    }

    fn open(&mut self) -> Result<(), QueryError> {
        // 初始化排序所需的任何资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.open()?;
        }
        Ok(())
    }

    fn close(&mut self) -> Result<(), QueryError> {
        // 清理资源
        if let Some(ref mut input_exec) = self.input_executor {
            input_exec.close()?;
        }
        Ok(())
    }

    fn id(&self) -> usize {
        self.base.id
    }

    fn name(&self) -> &str {
        &self.base.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value::{DataSet, Value};
    use crate::storage::StorageEngine;

    // #[tokio::test]
    // async fn test_single_column_sort() {
    //     let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
    //
    //     // 创建单列排序
    //     let sort_factors = vec![SortFactor::new("col1".to_string(), true)];
    //
    //     let executor = SortExecutor::new(1, storage, sort_factors);

    //     // 测试单列排序功能
    //     // 这里需要模拟输入数据
    // }

    // #[tokio::test]
    // async fn test_multi_column_sort() {
    //     let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
    //
    //     // 创建多列排序
    //     let sort_factors = vec![
    //         SortFactor::new("col1".to_string(), true),
    //         SortFactor::new("col2".to_string(), false),
    //     ];
    //
    //     let executor = SortExecutor::new(1, storage, sort_factors);

    //     // 测试多列排序功能
    //     // 这里需要模拟输入数据
    // }

    // #[tokio::test]
    // async fn test_descending_sort() {
    //     let storage = Arc::new(Mutex::new(MockStorageEngine::new()));
    //
    //     // 创建降序排序
    //     let sort_factors = vec![SortFactor::new("col1".to_string(), false)];
    //
    //     let executor = SortExecutor::new(1, storage, sort_factors);

    //     // 测试降序排序功能
    //     // 这里需要模拟输入数据
    // }
}