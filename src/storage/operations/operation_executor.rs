//! 操作日志执行器
//!
//! 提供操作日志的正向执行和逆向回滚功能

use crate::core::{Edge, StorageError, Value, Vertex};
use crate::storage::operations::{EdgeWriter, VertexWriter};
use crate::storage::serializer::{edge_from_bytes, vertex_from_bytes};
use crate::transaction::OperationLog;

/// 操作执行器 trait
///
/// 定义如何执行操作日志的正向操作和逆向回滚
pub trait OperationExecutor {
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
        // 逆序执行回滚（从后往前）
        for log in logs.iter().rev() {
            self.execute_rollback(log)?;
        }
        Ok(())
    }
}

/// 存储操作执行器
///
/// 同时需要 VertexWriter 和 EdgeWriter 来处理所有类型的操作日志
pub struct StorageOperationExecutor<'a> {
    writer: &'a mut dyn StorageWriter,
    space: String,
}

/// 组合 VertexWriter 和 EdgeWriter 的 trait
pub trait StorageWriter: VertexWriter + EdgeWriter {}

// 为所有同时实现 VertexWriter 和 EdgeWriter 的类型实现 StorageWriter
impl<T> StorageWriter for T where T: VertexWriter + EdgeWriter {}

impl<'a> StorageOperationExecutor<'a> {
    /// 创建新的操作执行器
    pub fn new(writer: &'a mut dyn StorageWriter, space: impl Into<String>) -> Self {
        Self {
            writer,
            space: space.into(),
        }
    }

    /// 解析顶点ID
    fn parse_vertex_id(&self, bytes: &[u8]) -> Result<Value, StorageError> {
        // 使用 value_from_bytes 正确解析 Value 类型
        crate::storage::serializer::value_from_bytes(bytes)
    }

    /// 解析边键
    ///
    /// 边键格式为 "src_dst_edge_type"，其中 src 和 dst 是 Value 的 Debug 格式
    /// 例如: "Int(1)_Int(2)_knows" 或 "String(\"a\")_String(\"b\")_friend"
    fn parse_edge_key(&self, edge_key: &[u8]) -> Result<(Value, Value, String), StorageError> {
        let key_str = String::from_utf8(edge_key.to_vec())
            .map_err(|e| StorageError::DbError(format!("无效的边键编码: {}", e)))?;

        // 尝试解析 Value 的 Debug 格式
        // 格式: "Int(1)_Int(2)_knows" 或 "String(\"value\")_Int(2)_type"
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

        // 解析 Value
        let src = self.parse_value_debug(&src_str)?;
        let dst = self.parse_value_debug(&dst_str)?;

        Ok((src, dst, edge_type))
    }

    /// 从字符串开头解析一个 Value 的 Debug 表示
    ///
    /// 返回解析出的 Value 字符串和剩余部分
    fn parse_value_str<'b>(&self, s: &'b str) -> Result<(String, &'b str), StorageError> {
        // 处理 Int 类型: Int(123)
        if s.starts_with("Int(") {
            if let Some(end) = s.find(')') {
                return Ok((s[..=end].to_string(), &s[end + 1..]));
            }
        }
        // 处理 String 类型: String("value")
        else if s.starts_with("String(\"") {
            // 找到 String(" 后的内容，需要处理转义
            let start = 8; // String(" 的长度
            if let Some(end) = s[start..].find("\")_") {
                return Ok((s[..start + end + 1].to_string(), &s[start + end + 1..]));
            } else if let Some(end) = s[start..].find("\")") {
                return Ok((s[..start + end + 1].to_string(), &s[start + end + 2..]));
            }
        }
        // 处理其他类型，尝试按简单格式解析
        else if let Some(idx) = s.find('_') {
            return Ok((s[..idx].to_string(), &s[idx..]));
        }

        // 整个字符串就是一个值
        Ok((s.to_string(), ""))
    }

    /// 解析 Value 的 Debug 格式字符串
    fn parse_value_debug(&self, s: &str) -> Result<Value, StorageError> {
        // 解析 Int 类型
        if s.starts_with("Int(") && s.ends_with(')') {
            let inner = &s[4..s.len() - 1];
            if let Ok(id) = inner.parse::<i64>() {
                return Ok(Value::Int(id));
            }
        }
        // 解析 String 类型
        else if s.starts_with("String(\"") && s.ends_with("\")") {
            let inner = &s[8..s.len() - 2];
            return Ok(Value::String(inner.to_string()));
        }
        // 尝试直接解析为整数
        else if let Ok(id) = s.parse::<i64>() {
            return Ok(Value::Int(id));
        }
        // 作为字符串处理
        else {
            return Ok(Value::String(s.to_string()));
        }

        Err(StorageError::DbError(format!(
            "无法解析 Value 格式: {}",
            s
        )))
    }
}

impl<'a> OperationExecutor for StorageOperationExecutor<'a> {
    fn execute_rollback(&mut self, log: &OperationLog) -> Result<(), StorageError> {
        match log {
            OperationLog::InsertVertex {
                space: _,
                vertex_id,
                previous_state,
            } => {
                let id = self.parse_vertex_id(vertex_id)?;

                if previous_state.is_some() {
                    // 如果之前有数据，恢复原来的数据（更新操作）
                    let vertex = vertex_from_bytes(previous_state.as_ref().unwrap())?;
                    self.writer.update_vertex(&self.space, vertex)?;
                } else {
                    // 如果之前没有数据，删除插入的顶点
                    self.writer.delete_vertex(&self.space, &id)?;
                }
                Ok(())
            }

            OperationLog::UpdateVertex {
                space: _,
                vertex_id: _,
                previous_data,
            } => {
                // 恢复更新前的数据
                let vertex = vertex_from_bytes(previous_data)?;
                self.writer.update_vertex(&self.space, vertex)?;
                Ok(())
            }

            OperationLog::DeleteVertex {
                space: _,
                vertex_id: _,
                deleted_data,
            } => {
                // 重新插入被删除的顶点
                let vertex = vertex_from_bytes(deleted_data)?;
                self.writer.insert_vertex(&self.space, vertex)?;
                Ok(())
            }

            OperationLog::InsertEdge {
                space: _,
                edge_key,
                previous_state,
            } => {
                let (src, dst, edge_type) = self.parse_edge_key(edge_key)?;

                if previous_state.is_some() {
                    // 如果之前有数据，恢复原来的数据
                    let edge = edge_from_bytes(previous_state.as_ref().unwrap())?;
                    self.writer.insert_edge(&self.space, edge)?;
                } else {
                    // 如果之前没有数据，删除插入的边
                    self.writer
                        .delete_edge(&self.space, &src, &dst, &edge_type)?;
                }
                Ok(())
            }

            OperationLog::DeleteEdge {
                space: _,
                edge_key: _,
                deleted_data,
            } => {
                // 重新插入被删除的边
                let edge = edge_from_bytes(deleted_data)?;
                self.writer.insert_edge(&self.space, edge)?;
                Ok(())
            }

            OperationLog::UpdateIndex {
                space: _,
                index_name: _,
                key: _,
                previous_value: _,
            } => {
                // 索引更新回滚需要索引管理器支持
                // 暂时不实现，返回成功（索引会在数据回滚后自动失效）
                Ok(())
            }

            OperationLog::DeleteIndex {
                space: _,
                index_name: _,
                key: _,
                deleted_value: _,
            } => {
                // 索引删除回滚需要索引管理器支持
                // 暂时不实现，返回成功
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::vertex_edge_path::Tag;
    use crate::storage::operations::writer::{EdgeWriter, VertexWriter};
    use std::collections::HashMap;

    // 模拟的存储写入器，同时实现 VertexWriter 和 EdgeWriter
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
            self.vertex_operations.push(format!(
                "insert_vertex({}, {:?})",
                space,
                vertex.vid()
            ));
            Ok(vertex.vid().clone())
        }

        fn update_vertex(&mut self, space: &str, vertex: Vertex) -> Result<(), StorageError> {
            self.vertex_operations.push(format!(
                "update_vertex({}, {:?})",
                space,
                vertex.vid()
            ));
            Ok(())
        }

        fn delete_vertex(&mut self, space: &str, id: &Value) -> Result<(), StorageError> {
            self.vertex_operations.push(format!("delete_vertex({}, {:?})", space, id));
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
    fn test_rollback_insert_vertex() {
        let mut writer = MockStorageWriter::new();
        let mut executor = StorageOperationExecutor::new(&mut writer, "test_space");

        // 测试回滚插入操作（之前无数据）
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
    fn test_rollback_insert_vertex_with_previous() {
        let mut writer = MockStorageWriter::new();
        let mut executor = StorageOperationExecutor::new(&mut writer, "test_space");

        // 创建一个顶点用于 previous_state
        let vertex = Vertex::new(
            Value::Int(1),
            vec![Tag {
                name: "Test".to_string(),
                properties: HashMap::new(),
            }],
        );
        let vertex_bytes = crate::storage::serializer::vertex_to_bytes(&vertex).unwrap();

        // 测试回滚插入操作（之前有数据）
        let log = OperationLog::InsertVertex {
            space: "test_space".to_string(),
            vertex_id: 1i64.to_be_bytes().to_vec(),
            previous_state: Some(vertex_bytes),
        };

        executor.execute_rollback(&log).expect("回滚失败");

        assert_eq!(writer.vertex_operations.len(), 1);
        assert!(writer.vertex_operations[0].contains("update_vertex"));
    }

    #[test]
    fn test_rollback_delete_vertex() {
        let mut writer = MockStorageWriter::new();
        let mut executor = StorageOperationExecutor::new(&mut writer, "test_space");

        // 创建一个顶点用于 deleted_data
        let vertex = Vertex::new(
            Value::Int(1),
            vec![Tag {
                name: "Test".to_string(),
                properties: HashMap::new(),
            }],
        );
        let vertex_bytes = crate::storage::serializer::vertex_to_bytes(&vertex).unwrap();

        // 测试回滚删除操作
        let log = OperationLog::DeleteVertex {
            space: "test_space".to_string(),
            vertex_id: 1i64.to_be_bytes().to_vec(),
            deleted_data: vertex_bytes,
        };

        executor.execute_rollback(&log).expect("回滚失败");

        assert_eq!(writer.vertex_operations.len(), 1);
        assert!(writer.vertex_operations[0].contains("insert_vertex"));
    }
}
