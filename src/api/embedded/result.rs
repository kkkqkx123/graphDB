//! 查询结果处理模块
//!
//! 提供完善的查询结果处理能力，扩展核心层的 QueryResult 和 Row

use crate::api::core::{CoreError, CoreResult, QueryResult as CoreQueryResult, Row as CoreRow};
use crate::core::{Value, Vertex, Edge, Path};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 查询结果
///
/// 封装核心层的查询结果，提供更方便的访问方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    columns: Vec<String>,
    rows: Vec<Row>,
    metadata: ResultMetadata,
}

/// 结果行
///
/// 封装核心层的行数据，提供按列名和索引访问的方法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    values: HashMap<String, Value>,
    column_index: HashMap<String, usize>,
}

/// 结果元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultMetadata {
    /// 执行时间
    pub execution_time: Duration,
    /// 返回行数
    pub rows_returned: usize,
    /// 扫描行数
    pub rows_scanned: u64,
}

impl QueryResult {
    /// 从核心层查询结果创建
    pub fn from_core(result: CoreQueryResult) -> Self {
        let columns = result.columns.clone();
        let rows: Vec<Row> = result.rows.into_iter().map(Row::from_core).collect();
        let rows_returned = rows.len();

        Self {
            columns: columns.clone(),
            rows,
            metadata: ResultMetadata {
                execution_time: Duration::from_millis(result.metadata.execution_time_ms),
                rows_returned,
                rows_scanned: result.metadata.rows_scanned,
            },
        }
    }

    /// 获取列名列表
    pub fn columns(&self) -> &[String] {
        &self.columns
    }

    /// 获取行数
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// 检查结果是否为空
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 获取指定行
    pub fn get(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    /// 获取第一行
    pub fn first(&self) -> Option<&Row> {
        self.rows.first()
    }

    /// 获取最后一行
    pub fn last(&self) -> Option<&Row> {
        self.rows.last()
    }

    /// 获取行迭代器
    pub fn iter(&self) -> impl Iterator<Item = &Row> {
        self.rows.iter()
    }

    /// 获取元数据
    pub fn metadata(&self) -> &ResultMetadata {
        &self.metadata
    }

    /// 获取所有行
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::Internal(format!("JSON序列化失败: {}", e)))
    }

    /// 转换为 JSON 字符串（紧凑格式）
    pub fn to_json_compact(&self) -> CoreResult<String> {
        serde_json::to_string(self)
            .map_err(|e| CoreError::Internal(format!("JSON序列化失败: {}", e)))
    }

    /// 转换为 JSON Value
    pub fn to_json_value(&self) -> CoreResult<serde_json::Value> {
        serde_json::to_value(self)
            .map_err(|e| CoreError::Internal(format!("JSON序列化失败: {}", e)))
    }
}

impl IntoIterator for QueryResult {
    type Item = Row;
    type IntoIter = std::vec::IntoIter<Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.into_iter()
    }
}

impl<'a> IntoIterator for &'a QueryResult {
    type Item = &'a Row;
    type IntoIter = std::slice::Iter<'a, Row>;

    fn into_iter(self) -> Self::IntoIter {
        self.rows.iter()
    }
}

impl Row {
    /// 从核心层行数据创建
    pub fn from_core(row: CoreRow) -> Self {
        let mut column_index = HashMap::new();
        let values = row.values;

        for (idx, (key, _)) in values.iter().enumerate() {
            column_index.insert(key.clone(), idx);
        }

        Self {
            values,
            column_index,
        }
    }

    /// 按列名获取值
    pub fn get(&self, column: &str) -> Option<&Value> {
        self.values.get(column)
    }

    /// 按索引获取值
    pub fn get_by_index(&self, index: usize) -> Option<&Value> {
        self.columns()
            .get(index)
            .and_then(|col| self.values.get(col.as_str()))
    }

    /// 获取所有列名
    pub fn columns(&self) -> Vec<&String> {
        self.values.keys().collect()
    }

    /// 获取列数
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// 检查是否为空行
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// 检查是否包含指定列
    pub fn has_column(&self, column: &str) -> bool {
        self.values.contains_key(column)
    }

    // 类型化获取方法

    /// 获取字符串值
    pub fn get_string(&self, column: &str) -> Option<String> {
        self.get(column).and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        })
    }

    /// 获取 i64 整数值
    pub fn get_int(&self, column: &str) -> Option<i64> {
        self.get(column).and_then(|v| match v {
            Value::Int(i) => Some(*i),
            _ => None,
        })
    }

    /// 获取 f64 浮点值
    pub fn get_float(&self, column: &str) -> Option<f64> {
        self.get(column).and_then(|v| match v {
            Value::Float(f) => Some(*f),
            _ => None,
        })
    }

    /// 获取布尔值
    pub fn get_bool(&self, column: &str) -> Option<bool> {
        self.get(column).and_then(|v| match v {
            Value::Bool(b) => Some(*b),
            _ => None,
        })
    }

    /// 获取顶点
    pub fn get_vertex(&self, column: &str) -> Option<&Vertex> {
        self.get(column).and_then(|v| match v {
            Value::Vertex(vertex) => Some(vertex.as_ref()),
            _ => None,
        })
    }

    /// 获取边
    pub fn get_edge(&self, column: &str) -> Option<&Edge> {
        self.get(column).and_then(|v| match v {
            Value::Edge(edge) => Some(edge),
            _ => None,
        })
    }

    /// 获取路径
    pub fn get_path(&self, column: &str) -> Option<&Path> {
        self.get(column).and_then(|v| match v {
            Value::Path(path) => Some(path),
            _ => None,
        })
    }

    /// 获取列表
    pub fn get_list(&self, column: &str) -> Option<&crate::core::value::dataset::List> {
        self.get(column).and_then(|v| match v {
            Value::List(list) => Some(list),
            _ => None,
        })
    }

    /// 获取映射
    pub fn get_map(&self, column: &str) -> Option<&HashMap<String, Value>> {
        self.get(column).and_then(|v| match v {
            Value::Map(map) => Some(map),
            _ => None,
        })
    }

    /// 获取所有值
    pub fn values(&self) -> &HashMap<String, Value> {
        &self.values
    }

    /// 转换为 JSON 字符串
    pub fn to_json(&self) -> CoreResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CoreError::Internal(format!("JSON序列化失败: {}", e)))
    }
}

impl Default for ResultMetadata {
    fn default() -> Self {
        Self {
            execution_time: Duration::from_millis(0),
            rows_returned: 0,
            rows_scanned: 0,
        }
    }
}

