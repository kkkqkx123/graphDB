//! 查询执行 API - 核心层
//!
//! 提供与传输层无关的查询执行功能

use crate::query::QueryPipelineManager;
use crate::storage::StorageClient;
use crate::api::core::{CoreResult, CoreError, QueryContext, QueryResult, Row, ExecutionMetadata};
use crate::core::StatsManager;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::Instant;

/// 通用查询 API - 核心层
pub struct QueryApi<S: StorageClient + 'static> {
    pipeline_manager: QueryPipelineManager<S>,
}

impl<S: StorageClient + Clone + 'static> QueryApi<S> {
    /// 创建新的 QueryApi 实例
    pub fn new(storage: Arc<S>) -> Self {
        let storage_mutex = Arc::new(Mutex::new((*storage).clone()));
        let stats_manager = Arc::new(StatsManager::new());
        Self {
            pipeline_manager: QueryPipelineManager::new(storage_mutex, stats_manager),
        }
    }

    /// 执行查询
    ///
    /// # 参数
    /// - `query`: 查询语句
    /// - `ctx`: 查询上下文
    ///
    /// # 返回
    /// 结构化查询结果
    pub async fn execute(
        &mut self,
        query: &str,
        ctx: QueryContext,
    ) -> CoreResult<QueryResult> {
        let start_time = Instant::now();

        // 构建空间信息
        let space_info = ctx.space_id.map(|id| crate::core::types::SpaceInfo {
            space_id: id,
            space_name: String::new(),
            vid_type: crate::core::DataType::String,
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: crate::core::types::MetadataVersion::default(),
            comment: None,
        });

        // 执行查询
        let execution_result = self
            .pipeline_manager
            .execute_query_with_space(query, space_info)
            .await
            .map_err(|e| CoreError::QueryExecutionFailed(e.to_string()))?;

        // 转换为结构化结果
        let mut result = Self::convert_to_query_result(execution_result)?;
        result.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(result)
    }

    /// 执行参数化查询
    pub async fn execute_with_params(
        &mut self,
        query: &str,
        params: std::collections::HashMap<String, crate::core::Value>,
        ctx: QueryContext,
    ) -> CoreResult<QueryResult> {
        let mut ctx = ctx;
        ctx.parameters = Some(params);
        self.execute(query, ctx).await
    }

    /// 将执行结果转换为结构化查询结果
    fn convert_to_query_result(
        execution: crate::query::executor::base::ExecutionResult,
    ) -> CoreResult<QueryResult> {
        match execution {
            crate::query::executor::base::ExecutionResult::DataSet(data) => {
                // 处理数据集结果 - DataSet 使用 col_names 而不是 columns
                let columns = data.col_names.clone();
                let mut rows = Vec::new();

                for row_data in &data.rows {
                    let mut row = Row::with_capacity(columns.len());
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(value) = row_data.get(i) {
                            row.insert(col.clone(), value.clone());
                        }
                    }
                    rows.push(row);
                }

                let metadata = ExecutionMetadata {
                    execution_time_ms: 0,
                    rows_scanned: data.rows.len() as u64,
                    rows_returned: data.rows.len() as u64,
                    cache_hit: false,
                };

                Ok(QueryResult {
                    columns,
                    rows,
                    metadata,
                })
            }
            crate::query::executor::base::ExecutionResult::Values(values) => {
                // 处理值列表结果
                let column = "value".to_string();
                let rows: Vec<Row> = values.into_iter().map(|v| {
                    let mut row = Row::new();
                    row.insert(column.clone(), v);
                    row
                }).collect();

                Ok(QueryResult {
                    columns: vec![column],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Vertices(vertices) => {
                // 处理顶点结果 - Value::Vertex 需要 Box<Vertex>
                let rows: Vec<Row> = vertices.into_iter().map(|v| {
                    let mut row = Row::new();
                    row.insert("vertex".to_string(), crate::core::Value::Vertex(Box::new(v)));
                    row
                }).collect();

                Ok(QueryResult {
                    columns: vec!["vertex".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Edges(edges) => {
                // 处理边结果 - Value::Edge 不需要 Box
                let rows: Vec<Row> = edges.into_iter().map(|e| {
                    let mut row = Row::new();
                    row.insert("edge".to_string(), crate::core::Value::Edge(e));
                    row
                }).collect();

                Ok(QueryResult {
                    columns: vec!["edge".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Result(core_result) => {
                // 处理 CoreResult - 使用 col_names() 方法和 rows() 方法
                let columns: Vec<String> = core_result.col_names().to_vec();
                let mut rows = Vec::new();

                for row_data in core_result.rows().iter() {
                    let mut row = Row::with_capacity(columns.len());
                    for (i, col) in columns.iter().enumerate() {
                        if let Some(value) = row_data.get(i) {
                            row.insert(col.clone(), value.clone());
                        }
                    }
                    rows.push(row);
                }

                Ok(QueryResult {
                    columns,
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Empty |
            crate::query::executor::base::ExecutionResult::Success => {
                // 空结果
                Ok(QueryResult {
                    columns: Vec::new(),
                    rows: Vec::new(),
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Count(count) => {
                // 计数结果
                let mut row = Row::new();
                row.insert("count".to_string(), crate::core::Value::Int(count as i64));

                Ok(QueryResult {
                    columns: vec!["count".to_string()],
                    rows: vec![row],
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Paths(paths) => {
                // 路径结果 - Value::Path 不需要 Box
                let rows: Vec<Row> = paths.into_iter().map(|p| {
                    let mut row = Row::new();
                    row.insert("path".to_string(), crate::core::Value::Path(p));
                    row
                }).collect();

                Ok(QueryResult {
                    columns: vec!["path".to_string()],
                    rows,
                    metadata: ExecutionMetadata::default(),
                })
            }
            crate::query::executor::base::ExecutionResult::Error(e) => {
                Err(CoreError::QueryExecutionFailed(e))
            }
        }
    }
}
