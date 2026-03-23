//! 操作日志回滚模块
//!
//! 提供基于操作日志的回滚功能，支持事务保存点回滚

use crate::core::{StorageError, Value};
use crate::core::vertex_edge_path::{Edge, Vertex};
use crate::storage::operations::{EdgeWriter, VertexWriter};
use crate::transaction::types::OperationLog;
use bincode::{config::standard, decode_from_slice, encode_to_vec};

/// 操作日志上下文 trait
///
/// 定义操作日志回滚所需的基本操作
pub trait OperationLogContext {
    /// 获取操作日志长度
    fn operation_log_len(&self) -> usize;
    /// 截断操作日志到指定索引
    fn truncate_operation_log(&self, index: usize);
    /// 获取指定索引的操作日志
    fn get_operation_log(&self, index: usize) -> Option<OperationLog>;
    /// 获取指定范围的操作日志
    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog>;
    /// 清空操作日志
    fn clear_operation_log(&self);
}

impl OperationLogContext for crate::transaction::context::TransactionContext {
    fn operation_log_len(&self) -> usize {
        self.operation_log_len()
    }

    fn truncate_operation_log(&self, index: usize) {
        self.truncate_operation_log(index);
    }

    fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
        self.get_operation_log(index)
    }

    fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
        self.get_operation_logs_range(start, end)
    }

    fn clear_operation_log(&self) {
        self.clear_operation_log();
    }
}

/// 回滚执行器 trait
///
/// 定义如何执行单个操作的逆操作
pub trait RollbackExecutor: Send {
    /// 执行单个操作日志的逆操作（回滚）
    ///
    /// # Arguments
    /// * `log` - 要回滚的操作日志
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(StorageError)` - 回滚失败
    fn execute_rollback(&mut self, log: &OperationLog) -> Result<(), StorageError>;

    /// 批量执行回滚操作
    ///
    /// 按逆序执行操作日志的回滚
    ///
    /// # Arguments
    /// * `logs` - 要回滚的操作日志列表
    ///
    /// # Returns
    /// * `Ok(())` - 回滚成功
    /// * `Err(StorageError)` - 回滚失败
    fn execute_rollback_batch(&mut self, logs: &[OperationLog]) -> Result<(), StorageError> {
        for log in logs.iter().rev() {
            self.execute_rollback(log)?;
        }
        Ok(())
    }
}

/// 组合 VertexWriter 和 EdgeWriter 的 trait
pub trait StorageWriter: VertexWriter + EdgeWriter {}

impl<T> StorageWriter for T where T: VertexWriter + EdgeWriter {}

/// 存储操作执行器
///
/// 基于 StorageWriter 实现操作日志的回滚功能
pub struct StorageRollbackExecutor<'a> {
    writer: &'a mut dyn StorageWriter,
    space: String,
}

impl<'a> StorageRollbackExecutor<'a> {
    /// 创建新的存储回滚执行器
    pub fn new(writer: &'a mut dyn StorageWriter, space: impl Into<String>) -> Self {
        Self {
            writer,
            space: space.into(),
        }
    }

    /// 解析顶点ID
    fn parse_vertex_id(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        decode_from_slice(bytes, standard())
            .map(|(v, _)| v)
            .map_err(|e| StorageError::DeserializeError(e.to_string()))
    }

    /// 解析边键
    fn parse_edge_key(&self, edge_key: &[u8]) -> Result<(Value, Value, String), StorageError> {
        let key_str = String::from_utf8(edge_key.to_vec())
            .map_err(|e| StorageError::DbError(format!("无效的边键编码: {}", e)))?;

        let (src_str, rest) = self.parse_value_str(&key_str)?;
        let rest = if rest.starts_with('_') {
            &rest[1..]
        } else {
            return Err(StorageError::DbError(format!(
                "无效的边键格式，缺少分隔符: {}",
                key_str
            )));
        };

        let (dst_str, edge_type) = self.parse_value_str(rest)?;
        let edge_type = if edge_type.starts_with('_') {
            edge_type[1..].to_string()
        } else {
            edge_type.to_string()
        };

        let src = self.parse_value_debug(&src_str)?;
        let dst = self.parse_value_debug(&dst_str)?;

        Ok((src, dst, edge_type))
    }

    /// 从字符串开头解析一个 Value 的 Debug 表示
    fn parse_value_str<'b>(&self, s: &'b str) -> Result<(String, &'b str), StorageError> {
        if s.starts_with("Int(") {
            if let Some(end) = s.find(')') {
                return Ok((s[..=end].to_string(), &s[end + 1..]));
            }
        } else if s.starts_with("String(\"") {
            let start = 8;
            if let Some(end) = s[start..].find("\")_") {
                return Ok((s[..start + end + 1].to_string(), &s[start + end + 1..]));
            } else if let Some(end) = s[start..].find("\")") {
                return Ok((s[..start + end + 1].to_string(), &s[start + end + 2..]));
            }
        } else if let Some(idx) = s.find('_') {
            return Ok((s[..idx].to_string(), &s[idx..]));
        }

        Ok((s.to_string(), ""))
    }

    /// 解析 Value 的 Debug 格式字符串
    fn parse_value_debug(&self, s: &str) -> Result<Value, StorageError> {
        if s.starts_with("Int(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            if let Ok(id) = inner.parse::<i64>() {
                return Ok(Value::Int(id));
            }
        } else if s.starts_with("String(\"") && s.ends_with("\")") {
            let inner = &s[8..s.len() - 2];
            return Ok(Value::String(inner.to_string()));
        } else if let Ok(id) = s.parse::<i64>() {
            return Ok(Value::Int(id));
        } else {
            return Ok(Value::String(s.to_string()));
        }

        Err(StorageError::DbError(format!("无法解析 Value 格式: {}", s)))
    }
}

impl<'a> RollbackExecutor for StorageRollbackExecutor<'a> {
    fn execute_rollback(&mut self, log: &OperationLog) -> Result<(), StorageError> {
        match log {
            OperationLog::InsertVertex {
                space: _,
                vertex_id,
                previous_state,
            } => {
                let id = self.parse_vertex_id(vertex_id)?;

                if let Some(ref state) = previous_state {
                    let vertex = decode_from_slice(state, standard())?.0;
                    self.writer.update_vertex(&self.space, vertex)?;
                } else {
                    self.writer.delete_vertex(&self.space, &id)?;
                }
                Ok(())
            }

            OperationLog::UpdateVertex {
                space: _,
                vertex_id: _,
                previous_data,
            } => {
                let vertex = decode_from_slice(previous_data, standard())?.0;
                self.writer.update_vertex(&self.space, vertex)?;
                Ok(())
            }

            OperationLog::DeleteVertex {
                space: _,
                vertex_id: _,
                vertex,
            } => {
                let decoded_vertex = decode_from_slice(vertex, standard())?.0;
                self.writer.insert_vertex(&self.space, decoded_vertex)?;
                Ok(())
            }

            OperationLog::InsertEdge {
                space: _,
                edge_id,
                previous_state,
            } => {
                let (src, dst, edge_type) = self.parse_edge_key(edge_id)?;

                if let Some(ref state) = previous_state {
                    let edge = decode_from_slice(state, standard())?.0;
                    self.writer.insert_edge(&self.space, edge)?;
                } else {
                    self.writer
                        .delete_edge(&self.space, &src, &dst, &edge_type)?;
                }
                Ok(())
            }

            OperationLog::DeleteEdge {
                space: _,
                edge_id: _,
                edge,
            } => {
                let decoded_edge = decode_from_slice(edge, standard())?.0;
                self.writer.insert_edge(&self.space, decoded_edge)?;
                Ok(())
            }

            OperationLog::UpdateEdge {
                space: _,
                edge_id: _,
                previous_data,
            } => {
                let edge = decode_from_slice(previous_data, standard())?.0;
                self.writer.insert_edge(&self.space, edge)?;
                Ok(())
            }
        }
    }
}

/// 操作日志回滚处理器
///
/// 负责执行基于操作日志的回滚操作
pub struct OperationLogRollback<'a, T: OperationLogContext> {
    ctx: &'a T,
}

impl<'a, T: OperationLogContext> OperationLogRollback<'a, T> {
    /// 创建新的回滚处理器
    pub fn new(ctx: &'a T) -> Self {
        Self { ctx }
    }

    /// 回滚到指定的操作日志索引
    pub fn rollback_to_index(&self, index: usize) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "无效的回滚索引: {}, 操作日志长度: {}",
                index, current_len
            )));
        }

        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// 使用执行器回滚到指定的操作日志索引
    pub fn execute_rollback_to_index<E: RollbackExecutor>(
        &self,
        index: usize,
        executor: &mut E,
    ) -> Result<(), StorageError> {
        let current_len = self.ctx.operation_log_len();

        if index > current_len {
            return Err(StorageError::DbError(format!(
                "无效的回滚索引: {}, 操作日志长度: {}",
                index, current_len
            )));
        }

        let logs_to_rollback = self.ctx.get_operation_logs(index, current_len);

        executor.execute_rollback_batch(&logs_to_rollback)?;

        self.ctx.truncate_operation_log(index);

        Ok(())
    }

    /// 获取操作日志长度
    pub fn operation_log_len(&self) -> usize {
        self.ctx.operation_log_len()
    }

    /// 获取所有操作日志
    pub fn get_all_logs(&self) -> Vec<OperationLog> {
        let len = self.ctx.operation_log_len();
        self.ctx.get_operation_logs(0, len)
    }

    /// 清空所有操作日志
    pub fn clear_logs(&self) {
        self.ctx.clear_operation_log();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::Tag;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct MockContext {
        logs: RefCell<Vec<OperationLog>>,
    }

    impl MockContext {
        fn new() -> Self {
            Self {
                logs: RefCell::new(Vec::new()),
            }
        }

        fn add_log(&self, log: OperationLog) {
            self.logs.borrow_mut().push(log);
        }
    }

    impl OperationLogContext for MockContext {
        fn operation_log_len(&self) -> usize {
            self.logs.borrow().len()
        }

        fn truncate_operation_log(&self, index: usize) {
            self.logs.borrow_mut().truncate(index);
        }

        fn get_operation_log(&self, index: usize) -> Option<OperationLog> {
            self.logs.borrow().get(index).cloned()
        }

        fn get_operation_logs(&self, start: usize, end: usize) -> Vec<OperationLog> {
            let logs = self.logs.borrow();
            if start >= logs.len() {
                return Vec::new();
            }
            let end = end.min(logs.len());
            logs[start..end].to_vec()
        }

        fn clear_operation_log(&self) {
            self.logs.borrow_mut().clear();
        }
    }

    struct MockStorageWriter {
        vertex_operations: Vec<String>,
        edge_operations: Vec<String>,
    }

    impl MockStorageWriter {
        fn new() -> Self {
            Self {
                vertex_operations: Vec::new(),
                edge_operations: Vec::new(),
            }
        }
    }

    impl VertexWriter for MockStorageWriter {
        fn insert_vertex(&mut self, space: &str, vertex: Vertex) -> Result<Value, StorageError> {
            self.vertex_operations
                .push(format!("insert_vertex({}, {:?})", space, vertex.vid()));
            Ok(vertex.vid().clone())
        }

        fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
            self.vertex_operations
                .push(format!("update_vertex({}, {:?})", space, vertex.vid()));
            Ok(())
        }

        fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
            self.vertex_operations
                .push(format!("delete_vertex({}, {:?})", space, id));
            Ok(())
        }

        fn batch_insert_vertices(
            &mut self,
            _space: &str,
            _vertices: Vec<Vertex>,
        ) -> Result<Vec<Value>, StorageError> {
            Ok(Vec::new())
        }

        fn delete_tags(
            &mut self,
            _space: &str,
            _vertex_id: &Value,
            _tag_names: &[String],
        ) -> Result<usize, StorageError> {
            Ok(0)
        }
    }

    impl EdgeWriter for MockStorageWriter {
        fn insert_edge(&mut self, space: &str, edge: Edge) -> Result<(), StorageError> {
            self.edge_operations.push(format!(
                "insert_edge({}, {:?}_{}_{})",
                space, edge.src, edge.dst, edge.edge_type
            ));
            Ok(())
        }

        fn delete_edge(
            &mut self,
            space: &str,
            src: &Value,
            dst: &Value,
            edge_type: &str,
        ) -> Result<(), StorageError> {
            self.edge_operations.push(format!(
                "delete_edge({}, {:?}_{}_{})",
                space, src, dst, edge_type
            ));
            Ok(())
        }

        fn batch_insert_edges(
            &mut self,
            _space: &str,
            _edges: Vec<Edge>,
        ) -> Result<(), StorageError> {
            Ok(())
        }
    }

    #[test]
    fn test_rollback_to_index() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_state: None,
        });

        ctx.add_log(OperationLog::UpdateVertex {
            space: "test".to_string(),
            vertex_id: vec![1, 2, 3],
            previous_data: vec![4, 5, 6],
        });

        assert_eq!(rollback.operation_log_len(), 2);

        let result = rollback.rollback_to_index(1);
        assert!(result.is_ok());
        assert_eq!(rollback.operation_log_len(), 1);
    }

    #[test]
    fn test_execute_rollback_with_executor() {
        let ctx = MockContext::new();
        let rollback = OperationLogRollback::new(&ctx);

        // 使用有效的顶点数据创建操作日志
        let vertex1 = Vertex::new(
            Value::Int(1),
            vec![Tag {
                name: "Test".to_string(),
                properties: HashMap::new(),
            }],
        );
        let vertex1_bytes = encode_to_vec(&vertex1, standard()).expect("顶点序列化失败");

        let vertex2 = Vertex::new(
            Value::Int(2),
            vec![Tag {
                name: "Test2".to_string(),
                properties: HashMap::new(),
            }],
        );
        let vertex2_bytes = encode_to_vec(&vertex2, standard()).expect("顶点序列化失败");

        ctx.add_log(OperationLog::InsertVertex {
            space: "test".to_string(),
            vertex_id: vertex1_bytes.clone(),
            previous_state: None,
        });

        ctx.add_log(OperationLog::UpdateVertex {
            space: "test".to_string(),
            vertex_id: vertex2_bytes.clone(),
            previous_data: vertex2_bytes,
        });

        let mut writer = MockStorageWriter::new();
        let mut executor = StorageRollbackExecutor::new(&mut writer, "test_space");

        let result = rollback.execute_rollback_to_index(0, &mut executor);
        assert!(result.is_ok());
        assert_eq!(rollback.operation_log_len(), 0);
    }

    #[test]
    fn test_rollback_insert_vertex() {
        let mut writer = MockStorageWriter::new();
        let mut executor = StorageRollbackExecutor::new(&mut writer, "test_space");

        let log = OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: 1i64.to_be_bytes().to_vec(),
            previous_state: None,
        };

        executor.execute_rollback(&log).expect("回滚失败");

        assert_eq!(writer.vertex_operations.len(), 1);
        assert!(writer.vertex_operations[0].contains("delete_vertex"));
    }

    #[test]
    fn test_rollback_delete_vertex() {
        let mut writer = MockStorageWriter::new();
        let mut executor = StorageRollbackExecutor::new(&mut writer, "test_space");

        let vertex = Vertex::new(
            Value::Int(1),
            vec![Tag {
                name: "Test".to_string(),
                properties: HashMap::new(),
            }],
        );
        let vertex_bytes = encode_to_vec(&vertex, standard()).expect("顶点序列化失败");

        let log = OperationLog::DeleteVertex {
            space: "test_space".to_string(),
            vertex_id: 1i64.to_be_bytes().to_vec(),
            vertex: vertex_bytes,
        };

        executor.execute_rollback(&log).expect("回滚失败");

        assert_eq!(writer.vertex_operations.len(), 1);
        assert!(writer.vertex_operations[0].contains("insert_vertex"));
    }
}
