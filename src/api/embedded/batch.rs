//! 批量操作模块
//!
//! 支持高效的大批量数据导入

use crate::api::core::{CoreError, CoreResult};
use crate::api::embedded::session::Session;
use crate::core::{Edge, Vertex};
use crate::storage::StorageClient;

/// 批量插入器
///
/// 用于高效地批量插入顶点和边数据
///
/// # 示例
///
/// ```rust
/// use graphdb::api::embedded::{GraphDatabase, DatabaseConfig};
/// use graphdb::core::{Vertex, Edge, Value};
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let db = GraphDatabase::open("my_db")?;
/// let session = db.session()?;
///
/// // 创建批量插入器，每100条自动刷新
/// let mut inserter = session.batch_inserter(100);
///
/// // 添加顶点
/// for i in 0..1000 {
///     let vertex = Vertex::with_vid(Value::Int(i));
///     inserter.add_vertex(vertex);
/// }
///
/// // 执行批量插入
/// let result = inserter.execute()?;
/// println!("插入了 {} 个顶点", result.vertices_inserted);
/// # Ok(())
/// # }
/// ```
pub struct BatchInserter<'sess, S: StorageClient + Clone + 'static> {
    session: &'sess Session<S>,
    batch_size: usize,
    vertex_buffer: Vec<Vertex>,
    edge_buffer: Vec<Edge>,
    total_inserted: BatchResult,
}

/// 批量操作结果
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// 插入的顶点数量
    pub vertices_inserted: usize,
    /// 插入的边数量
    pub edges_inserted: usize,
    /// 错误列表
    pub errors: Vec<BatchError>,
}

/// 批量错误
#[derive(Debug, Clone)]
pub struct BatchError {
    /// 错误发生的索引
    pub index: usize,
    /// 错误项类型
    pub item_type: BatchItemType,
    /// 错误信息
    pub error: String,
}

/// 批量项类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchItemType {
    /// 顶点
    Vertex,
    /// 边
    Edge,
}

impl<'sess, S: StorageClient + Clone + 'static> BatchInserter<'sess, S> {
    /// 创建新的批量插入器
    pub(crate) fn new(session: &'sess Session<S>, batch_size: usize) -> Self {
        Self {
            session,
            batch_size: batch_size.max(1), // 确保至少为1
            vertex_buffer: Vec::with_capacity(batch_size),
            edge_buffer: Vec::with_capacity(batch_size),
            total_inserted: BatchResult {
                vertices_inserted: 0,
                edges_inserted: 0,
                errors: Vec::new(),
            },
        }
    }

    /// 添加顶点
    ///
    /// # 参数
    /// - `vertex` - 要插入的顶点
    ///
    /// # 返回
    /// - 返回自身，支持链式调用
    pub fn add_vertex(&mut self, vertex: Vertex) -> &mut Self {
        self.vertex_buffer.push(vertex);

        // 如果达到批次大小，自动刷新
        if self.vertex_buffer.len() >= self.batch_size {
            let _ = self.flush_vertices();
        }

        self
    }

    /// 添加边
    ///
    /// # 参数
    /// - `edge` - 要插入的边
    ///
    /// # 返回
    /// - 返回自身，支持链式调用
    pub fn add_edge(&mut self, edge: Edge) -> &mut Self {
        self.edge_buffer.push(edge);

        // 如果达到批次大小，自动刷新
        if self.edge_buffer.len() >= self.batch_size {
            let _ = self.flush_edges();
        }

        self
    }

    /// 添加多个顶点
    ///
    /// # 参数
    /// - `vertices` - 要插入的顶点列表
    pub fn add_vertices(&mut self, vertices: Vec<Vertex>) -> &mut Self {
        for vertex in vertices {
            self.add_vertex(vertex);
        }
        self
    }

    /// 添加多个边
    ///
    /// # 参数
    /// - `edges` - 要插入的边列表
    pub fn add_edges(&mut self, edges: Vec<Edge>) -> &mut Self {
        for edge in edges {
            self.add_edge(edge);
        }
        self
    }

    /// 执行批量插入
    ///
    /// 刷新所有缓冲的数据并返回结果
    ///
    /// # 返回
    /// - 成功时返回批量操作结果
    /// - 失败时返回错误
    pub fn execute(mut self) -> CoreResult<BatchResult> {
        // 刷新剩余的顶点
        self.flush_vertices()?;

        // 刷新剩余的边
        self.flush_edges()?;

        Ok(self.total_inserted)
    }

    /// 刷新顶点缓冲区
    fn flush_vertices(&mut self) -> CoreResult<()> {
        if self.vertex_buffer.is_empty() {
            return Ok(());
        }

        // 获取当前空间名称
        let space_name = self.session.space_name()
            .ok_or_else(|| CoreError::InvalidParameter("未选择图空间".to_string()))?;

        // 取出缓冲区中的顶点
        let vertices_to_insert: Vec<Vertex> = std::mem::take(&mut self.vertex_buffer);
        let count = vertices_to_insert.len();

        // 调用存储层的批量插入接口
        let mut storage = self.session.storage();
        match storage.batch_insert_vertices(space_name, vertices_to_insert) {
            Ok(_) => {
                // 插入成功，更新计数
                self.total_inserted.vertices_inserted += count;
            }
            Err(e) => {
                // 插入失败，记录错误，但不立即返回错误
                // 这样调用者可以通过 BatchResult 获取部分成功的结果和所有错误
                self.total_inserted.errors.push(BatchError {
                    index: self.total_inserted.vertices_inserted,
                    item_type: BatchItemType::Vertex,
                    error: format!("批量插入顶点失败: {}", e),
                });
            }
        }

        Ok(())
    }

    /// 刷新边缓冲区
    fn flush_edges(&mut self) -> CoreResult<()> {
        if self.edge_buffer.is_empty() {
            return Ok(());
        }

        // 获取当前空间名称
        let space_name = self.session.space_name()
            .ok_or_else(|| CoreError::InvalidParameter("未选择图空间".to_string()))?;

        // 取出缓冲区中的边
        let edges_to_insert: Vec<Edge> = std::mem::take(&mut self.edge_buffer);
        let count = edges_to_insert.len();

        // 调用存储层的批量插入接口
        let mut storage = self.session.storage();
        match storage.batch_insert_edges(space_name, edges_to_insert) {
            Ok(_) => {
                // 插入成功，更新计数
                self.total_inserted.edges_inserted += count;
            }
            Err(e) => {
                // 插入失败，记录错误，但不立即返回错误
                // 这样调用者可以通过 BatchResult 获取部分成功的结果和所有错误
                self.total_inserted.errors.push(BatchError {
                    index: self.total_inserted.edges_inserted,
                    item_type: BatchItemType::Edge,
                    error: format!("批量插入边失败: {}", e),
                });
            }
        }

        Ok(())
    }

    /// 获取当前缓冲区中的顶点数量
    pub fn buffered_vertices(&self) -> usize {
        self.vertex_buffer.len()
    }

    /// 获取当前缓冲区中的边数量
    pub fn buffered_edges(&self) -> usize {
        self.edge_buffer.len()
    }

    /// 获取批次大小
    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    /// 检查是否有缓冲的数据
    pub fn has_buffered_data(&self) -> bool {
        !self.vertex_buffer.is_empty() || !self.edge_buffer.is_empty()
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self {
            vertices_inserted: 0,
            edges_inserted: 0,
            errors: Vec::new(),
        }
    }
}

impl BatchResult {
    /// 获取总插入数量
    pub fn total_inserted(&self) -> usize {
        self.vertices_inserted + self.edges_inserted
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }

    /// 合并另一个批量结果
    pub fn merge(&mut self, other: BatchResult) {
        self.vertices_inserted += other.vertices_inserted;
        self.edges_inserted += other.edges_inserted;
        self.errors.extend(other.errors);
    }
}

impl BatchError {
    /// 创建新的批量错误
    pub fn new(index: usize, item_type: BatchItemType, error: impl Into<String>) -> Self {
        Self {
            index,
            item_type,
            error: error.into(),
        }
    }
}

/// 批量操作配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批次大小
    pub batch_size: usize,
    /// 是否自动提交
    pub auto_commit: bool,
    /// 是否忽略错误继续处理
    pub continue_on_error: bool,
    /// 最大错误数量
    pub max_errors: Option<usize>,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1000,
            auto_commit: true,
            continue_on_error: true,
            max_errors: Some(100),
        }
    }
}

impl BatchConfig {
    /// 创建新的配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置批次大小
    pub fn with_batch_size(mut self, size: usize) -> Self {
        self.batch_size = size.max(1);
        self
    }

    /// 设置是否自动提交
    pub fn with_auto_commit(mut self, auto_commit: bool) -> Self {
        self.auto_commit = auto_commit;
        self
    }

    /// 设置是否继续处理错误
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// 设置最大错误数量
    pub fn with_max_errors(mut self, max_errors: Option<usize>) -> Self {
        self.max_errors = max_errors;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_result_default() {
        let result = BatchResult::default();
        assert_eq!(result.vertices_inserted, 0);
        assert_eq!(result.edges_inserted, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_batch_result_total_inserted() {
        let result = BatchResult {
            vertices_inserted: 100,
            edges_inserted: 50,
            errors: Vec::new(),
        };
        assert_eq!(result.total_inserted(), 150);
    }

    #[test]
    fn test_batch_result_merge() {
        let mut result1 = BatchResult {
            vertices_inserted: 100,
            edges_inserted: 50,
            errors: vec![BatchError::new(0, BatchItemType::Vertex, "error1")],
        };

        let result2 = BatchResult {
            vertices_inserted: 200,
            edges_inserted: 100,
            errors: vec![BatchError::new(1, BatchItemType::Edge, "error2")],
        };

        result1.merge(result2);

        assert_eq!(result1.vertices_inserted, 300);
        assert_eq!(result1.edges_inserted, 150);
        assert_eq!(result1.errors.len(), 2);
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 1000);
        assert!(config.auto_commit);
        assert!(config.continue_on_error);
        assert_eq!(config.max_errors, Some(100));
    }

    #[test]
    fn test_batch_config_builder() {
        let config = BatchConfig::new()
            .with_batch_size(500)
            .with_auto_commit(false)
            .with_continue_on_error(false)
            .with_max_errors(Some(50));

        assert_eq!(config.batch_size, 500);
        assert!(!config.auto_commit);
        assert!(!config.continue_on_error);
        assert_eq!(config.max_errors, Some(50));
    }

    #[test]
    fn test_batch_config_min_batch_size() {
        let config = BatchConfig::new().with_batch_size(0);
        assert_eq!(config.batch_size, 1); // 最小值为1
    }
}
