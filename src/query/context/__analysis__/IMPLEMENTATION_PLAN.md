# GraphDB 上下文模块修改方案

## 概述

本文档详细描述 `query/context` 模块的改进方案，包括具体的修改内容、实施步骤和预期效果。修改方案基于 `__analysis__/` 目录下的分析文档制定。

---

## 一、修改优先级

### 1.1 高优先级（必须实施）

| 序号 | 修改项 | 预期效果 | 工作量 |
|------|--------|---------|--------|
| 1 | 引入强类型 DataType 枚举 | 提高类型安全性 | 中 |
| 2 | 统一错误处理系统 | 完善错误处理 | 中 |
| 3 | 实现 ResultIterator 迭代器 | 支持流式数据处理 | 大 |

### 1.2 中优先级（建议实施）

| 序号 | 修改项 | 预期效果 | 工作量 |
|------|--------|---------|--------|
| 4 | 拆分 QueryContext 职责 | 降低耦合度 | 大 |
| 5 | 添加 ExpressionContext | 支持复杂表达式 | 中 |
| 6 | 重构 SchemaManager 接口 | 降低实现复杂度 | 中 |

### 1.3 低优先级（可选实施）

| 序号 | 修改项 | 预期效果 | 工作量 |
|------|--------|---------|--------|
| 7 | 优化并发模型 | 提高性能 | 中 |
| 8 | 增强生成器作用域管理 | 支持嵌套查询 | 小 |

---

## 二、高优先级修改详细方案

### 2.1 修改 1：引入强类型 DataType 枚举

#### 2.1.1 修改文件

- `src/query/context/validate/types.rs`

#### 2.1.2 当前问题

```rust
// 当前实现
pub struct Column {
    pub name: String,
    pub type_: String,  // 使用字符串表示类型
}
```

#### 2.1.3 修改方案

```rust
// 新增 DataType 枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataType {
    // 基础类型
    Int64,
    Int32,
    Int16,
    Float,
    Double,
    Bool,
    String,
    Date,
    DateTime,
    Time,
    Timestamp,
    Geography,

    // 复合类型
    List(Box<DataType>),
    Set(Box<DataType>),
    Map(Box<DataType>, Box<DataType>),
    Struct(Vec<(String, DataType)>),

    // 图类型
    Vertex,
    Edge,
    Path,

    // 特殊类型
    DataSet,
    Null,
    Unknown,
}

// 实现 Display trait 用于错误信息
impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataType::Int64 => write!(f, "INT64"),
            DataType::Int32 => write!(f, "INT32"),
            DataType::Int16 => write!(f, "INT16"),
            DataType::Float => write!(f, "FLOAT"),
            DataType::Double => write!(f, "DOUBLE"),
            DataType::Bool => write!(f, "BOOL"),
            DataType::String => write!(f, "STRING"),
            DataType::Date => write!(f, "DATE"),
            DataType::DateTime => write!(f, "DATETIME"),
            DataType::Time => write!(f, "TIME"),
            DataType::Timestamp => write!(f, "TIMESTAMP"),
            DataType::Geography => write!(f, "GEOGRAPHY"),
            DataType::List(inner) => write!(f, "LIST<{}>", inner),
            DataType::Set(inner) => write!(f, "SET<{}>", inner),
            DataType::Map(k, v) => write!(f, "MAP<{}, {}>", k, v),
            DataType::Struct(fields) => {
                let field_strs: Vec<String> = fields
                    .iter()
                    .map(|(n, t)| format!("{}: {}", n, t))
                    .collect();
                write!(f, "STRUCT<{}>", field_strs.join(", "))
            }
            DataType::Vertex => write!(f, "VERTEX"),
            DataType::Edge => write!(f, "EDGE"),
            DataType::Path => write!(f, "PATH"),
            DataType::DataSet => write!(f, "DATASET"),
            DataType::Null => write!(f, "NULL"),
            DataType::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

// 新增 Column 类型
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub type_: DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub comment: Option<String>,
}

// 更新 SpaceInfo
#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub space_name: String,
    pub space_id: Option<u32>,
    pub is_default: bool,
    pub vid_type: DataType,  // 顶点ID类型
}

// 添加类型转换辅助函数
impl DataType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "INT64" | "INT" | "INTEGER" => Some(DataType::Int64),
            "INT32" | "SMALLINT" => Some(DataType::Int32),
            "INT16" | "TINYINT" => Some(DataType::Int16),
            "FLOAT" => Some(DataType::Float),
            "DOUBLE" => Some(DataType::Double),
            "BOOL" | "BOOLEAN" => Some(DataType::Bool),
            "STRING" | "VARCHAR" => Some(DataType::String),
            "DATE" => Some(DataType::Date),
            "DATETIME" => Some(DataType::DateTime),
            "TIME" => Some(DataType::Time),
            "TIMESTAMP" => Some(DataType::Timestamp),
            "VERTEX" => Some(DataType::Vertex),
            "EDGE" => Some(DataType::Edge),
            "PATH" => Some(DataType::Path),
            "DATASET" | "DATA_SET" => Some(DataType::DataSet),
            "NULL" => Some(DataType::Null),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            DataType::Int64 | DataType::Int32 | DataType::Int16 |
            DataType::Float | DataType::Double
        )
    }

    pub fn is_compatible_with(&self, other: &DataType) -> bool {
        // 简化版兼容性检查
        match (self, other) {
            (DataType::Int64, DataType::Int32) => true,
            (DataType::Int32, DataType::Int64) => true,
            (DataType::Float, DataType::Double) => true,
            (DataType::Double, DataType::Float) => true,
            (a, b) => a == b,
        }
    }
}
```

#### 2.1.4 依赖修改

需要更新以下文件以使用新的 `DataType`：
- `src/query/context/validate/schema.rs` - `SchemaInfo` 和 `FieldInfo`
- `src/query/context/symbol/symbol_table.rs` - `Symbol` 结构
- `src/core/types.rs` - 核心类型定义

#### 2.1.5 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_type_from_str() {
        assert_eq!(DataType::from_str("INT"), Some(DataType::Int64));
        assert_eq!(DataType::from_str("STRING"), Some(DataType::String));
        assert_eq!(DataType::from_str("UNKNOWN"), None);
    }

    #[test]
    fn test_data_type_display() {
        assert_eq!(DataType::Int64.to_string(), "INT64");
        assert_eq!(DataType::String.to_string(), "STRING");
        assert_eq!(DataType::List(Box::new(DataType::Int64)).to_string(), "LIST<INT64>");
    }

    #[test]
    fn test_data_type_compatibility() {
        assert!(DataType::Int64.is_compatible_with(&DataType::Int32));
        assert!(!DataType::Int64.is_compatible_with(&DataType::String));
    }
}
```

---

### 2.2 修改 2：统一错误处理系统

#### 2.2.1 修改文件

- 新建 `src/core/error/query_error.rs`
- 修改 `src/core/error/mod.rs`

#### 2.2.2 修改方案

```rust
// src/core/error/query_error.rs

use thiserror::Error;
use std::fmt;

// 错误码枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ErrorCode {
    // 成功
    Success = 0,

    // 通用错误 (1-99)
    Failed = 1,
    E_UNKNOWN = 2,

    // 语法错误 (1000-1999)
    SyntaxError = 1001,
    UnterminatedString = 1002,
    InvalidNumber = 1003,
    UnexpectedToken = 1004,
    MissingToken = 1005,

    // 语义错误 (2000-2999)
    SpaceNotFound = 2001,
    TagNotFound = 2002,
    EdgeTypeNotFound = 2003,
    PropertyNotFound = 2004,
    TypeMismatch = 2005,
    VariableNotFound = 2006,
    AmbiguousVariable = 2007,
    InvalidDataType = 2008,

    // 执行错误 (3000-3999)
    ExecutionError = 3001,
    StorageError = 3002,
    MemoryExceeded = 3003,
    TimeoutError = 3004,
    PartialSuccess = 3005,

    // 权限错误 (4000-4999)
    PermissionDenied = 4001,
    AuthenticationFailed = 4002,

    // 事务错误 (5000-5999)
    TransactionConflict = 5001,
    TransactionAborted = 5002,
    TransactionTimeout = 5003,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::Success => write!(f, "Success"),
            ErrorCode::Failed => write!(f, "Failed"),
            ErrorCode::E_UNKNOWN => write!(f, "Unknown error"),
            ErrorCode::SyntaxError => write!(f, "Syntax error"),
            ErrorCode::UnterminatedString => write!(f, "Unterminated string"),
            ErrorCode::InvalidNumber => write!(f, "Invalid number format"),
            ErrorCode::UnexpectedToken => write!(f, "Unexpected token"),
            ErrorCode::MissingToken => write!(f, "Missing token"),
            ErrorCode::SpaceNotFound => write!(f, "Space not found"),
            ErrorCode::TagNotFound => write!(f, "Tag not found"),
            ErrorCode::EdgeTypeNotFound => write!(f, "Edge type not found"),
            ErrorCode::PropertyNotFound => write!(f, "Property not found"),
            ErrorCode::TypeMismatch => write!(f, "Type mismatch"),
            ErrorCode::VariableNotFound => write!(f, "Variable not found"),
            ErrorCode::AmbiguousVariable => write!(f, "Ambiguous variable"),
            ErrorCode::InvalidDataType => write!(f, "Invalid data type"),
            ErrorCode::ExecutionError => write!(f, "Execution error"),
            ErrorCode::StorageError => write!(f, "Storage error"),
            ErrorCode::MemoryExceeded => write!(f, "Memory limit exceeded"),
            ErrorCode::TimeoutError => write!(f, "Query timeout"),
            ErrorCode::PartialSuccess => write!(f, "Partial success"),
            ErrorCode::PermissionDenied => write!(f, "Permission denied"),
            ErrorCode::AuthenticationFailed => write!(f, "Authentication failed"),
            ErrorCode::TransactionConflict => write!(f, "Transaction conflict"),
            ErrorCode::TransactionAborted => write!(f, "Transaction aborted"),
            ErrorCode::TransactionTimeout => write!(f, "Transaction timeout"),
        }
    }
}

// 查询错误类型
#[derive(Debug, Error)]
pub enum QueryError {
    #[error("语法错误: {message}")]
    SyntaxError {
        message: String,
        line: Option<u32>,
        column: Option<u32>,
    },

    #[error("语义验证失败: {message}")]
    ValidationError {
        message: String,
        code: ErrorCode,
        context: Option<String>,
    },

    #[error("执行计划错误: {message}")]
    PlanningError {
        message: String,
        code: ErrorCode,
    },

    #[error("执行错误 (代码: {code}): {message}")]
    ExecutionError {
        code: ErrorCode,
        message: String,
        retryable: bool,
    },

    #[error("Schema 错误: {message}")]
    SchemaError {
        code: ErrorCode,
        message: String,
        schema_name: Option<String>,
    },

    #[error("存储层错误: {message}")]
    StorageError {
        code: ErrorCode,
        message: String,
        storage_partition: Option<i32>,
    },

    #[error("事务错误: {message}")]
    TransactionError {
        code: ErrorCode,
        message: String,
        transaction_id: Option<i64>,
    },

    #[error("权限错误: {message}")]
    PermissionError {
        code: ErrorCode,
        message: String,
        operation: String,
        resource: String,
    },

    #[error("超时: 查询执行超过 {timeout_ms}ms")]
    TimeoutError {
        timeout_ms: u64,
        query: String,
    },

    #[error("查询被取消")]
    CancelledError,

    #[error("内部错误: {message}")]
    InternalError {
        message: String,
        location: &'static str,
    },
}

// 实现 From 转换
impl From<QueryError> for std::io::Error {
    fn from(err: QueryError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
    }
}

// 结果类型别名
pub type QueryResult<T> = Result<T, QueryError>;

// 辅助函数
impl QueryError {
    pub fn syntax_error(message: impl Into<String>) -> Self {
        QueryError::SyntaxError {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    pub fn validation_error(message: impl Into<String>, code: ErrorCode) -> Self {
        QueryError::ValidationError {
            message: message.into(),
            code,
            context: None,
        }
    }

    pub fn execution_error(message: impl Into<String>, code: ErrorCode) -> Self {
        QueryError::ExecutionError {
            code,
            message: message.into(),
            retryable: matches!(code, ErrorCode::StorageError),
        }
    }

    pub fn not_found_error(resource: impl Into<String>, name: impl Into<String>) -> Self {
        let code = match resource.into().as_str() {
            "space" => ErrorCode::SpaceNotFound,
            "tag" => ErrorCode::TagNotFound,
            "edge" => ErrorCode::EdgeTypeNotFound,
            "property" => ErrorCode::PropertyNotFound,
            "variable" => ErrorCode::VariableNotFound,
            _ => ErrorCode::Failed,
        };
        QueryError::ValidationError {
            message: format!("{} not found: {}", resource.into(), name.into()),
            code,
            context: None,
        }
    }

    pub fn with_location(mut self, location: &'static str) -> Self {
        if let QueryError::InternalError { message, .. } = &mut self {
            *message = format!("{} (at {})", message, location);
        }
        self
    }
}

// 验证错误详情
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub field: Option<String>,
    pub suggestion: Option<String>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        if let Some(field) = &self.field {
            write!(f, " (field: {})", field)?;
        }
        if let Some(suggestion) = &self.suggestion {
            write!(f, ". Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}
```

#### 2.2.3 更新现有错误处理

修改 `src/core/error/mod.rs`:

```rust
pub mod query_error;

pub use query_error::{
    ErrorCode, QueryError, QueryResult, ValidationError,
};
```

#### 2.2.4 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::Success.to_string(), "Success");
        assert_eq!(ErrorCode::SpaceNotFound.to_string(), "Space not found");
    }

    #[test]
    fn test_query_error_creation() {
        let err = QueryError::syntax_error("Unexpected token");
        assert!(err.to_string().contains("语法错误"));

        let err = QueryError::not_found_error("space", "test_space");
        assert!(err.to_string().contains("Space not found"));
    }

    #[test]
    fn test_query_error_conversion() {
        let result: QueryResult<i32> = Err(QueryError::execution_error("test", ErrorCode::StorageError));
        assert!(result.is_err());
    }
}
```

---

### 2.3 修改 3：实现 ResultIterator 迭代器系统

#### 2.3.1 修改文件

- 新建 `src/query/context/execution/iterator.rs`

#### 2.3.2 修改方案

```rust
// src/query/context/execution/iterator.rs

use crate::core::{Value, Row};
use std::fmt;

/// 迭代器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorKind {
    Default,
    Sequential,
    GetNeighbors,
    Prop,
    Join,
    Set,
}

/// 迭代器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IteratorState {
    Uninitialized,
    Ready,
    Iterating,
    Finished,
}

/// 迭代器 trait
pub trait ResultIterator: Send {
    /// 获取迭代器类型
    fn kind(&self) -> IteratorKind;

    /// 获取当前状态
    fn state(&self) -> IteratorState;

    /// 检查是否有效
    fn valid(&self) -> bool;

    /// 移动到下一个元素
    fn next(&mut self);

    /// 获取当前行的值
    fn value(&self) -> Option<&Row>;

    /// 获取当前行的所有权
    fn into_value(self: Box<Self>) -> Option<Row>;

    /// 重置迭代器
    fn reset(&mut self);

    /// 获取迭代器大小
    fn size(&self) -> usize;

    /// 获取列名
    fn column_names(&self) -> &[String];

    /// 检查内存
    fn check_memory(&self) -> bool;

    /// 设置内存检查
    fn set_check_memory(&mut self, check: bool);
}

/// 顺序迭代器
#[derive(Debug)]
pub struct SequentialIter {
    state: IteratorState,
    data: Vec<Row>,
    index: usize,
    column_names: Vec<String>,
    check_memory: bool,
}

impl SequentialIter {
    pub fn new(data: Vec<Row>, column_names: Vec<String>) -> Self {
        Self {
            state: if data.is_empty() {
                IteratorState::Finished
            } else {
                IteratorState::Ready
            },
            data,
            index: 0,
            column_names,
            check_memory: false,
        }
    }

    pub fn from_value(value: Value) -> Option<Self> {
        match value {
            Value::DataSet(dataset) => {
                Some(Self::new(dataset.rows, dataset.columns))
            }
            _ => None,
        }
    }
}

impl ResultIterator for SequentialIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Sequential
    }

    fn state(&self) -> IteratorState {
        self.state
    }

    fn valid(&self) -> bool {
        self.index < self.data.len()
    }

    fn next(&mut self) {
        if self.valid() {
            self.index += 1;
        }
        if !self.valid() {
            self.state = IteratorState::Finished;
        }
    }

    fn value(&self) -> Option<&Row> {
        self.data.get(self.index)
    }

    fn into_value(mut self: Box<Self>) -> Option<Row> {
        self.data.pop()
    }

    fn reset(&mut self) {
        self.index = 0;
        self.state = if self.data.is_empty() {
            IteratorState::Finished
        } else {
            IteratorState::Ready
        };
    }

    fn size(&self) -> usize {
        self.data.len()
    }

    fn column_names(&self) -> &[String] {
        &self.column_names
    }

    fn check_memory(&self) -> bool {
        self.check_memory
    }

    fn set_check_memory(&mut self, check: bool) {
        self.check_memory = check;
    }
}

/// 邻居遍历迭代器
#[derive(Debug)]
pub struct GetNeighborsIter {
    state: IteratorState,
    vertices: Vec<Value>,
    current_vertex: Option<Value>,
    edges: Vec<(Value, Vec<Row>)>,
    edge_index: usize,
    row_index: usize,
    column_names: Vec<String>,
    check_memory: bool,
}

impl GetNeighborsIter {
    pub fn new(vertices: Vec<Value>, column_names: Vec<String>) -> Self {
        Self {
            state: if vertices.is_empty() {
                IteratorState::Finished
            } else {
                IteratorState::Ready
            },
            vertices,
            current_vertex: None,
            edges: Vec::new(),
            edge_index: 0,
            row_index: 0,
            column_names,
            check_memory: false,
        }
    }

    pub fn add_neighbors(&mut self, vertex: Value, neighbors: Vec<Row>) {
        self.edges.push((vertex, neighbors));
    }
}

impl ResultIterator for GetNeighborsIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::GetNeighbors
    }

    fn state(&self) -> IteratorState {
        self.state
    }

    fn valid(&self) -> bool {
        self.edge_index < self.edges.len()
    }

    fn next(&mut self) {
        self.row_index += 1;
        while self.edge_index < self.edges.len() {
            if self.row_index < self.edges[self.edge_index].1.len() {
                return;
            }
            self.edge_index += 1;
            self.row_index = 0;
        }
        self.state = IteratorState::Finished;
    }

    fn value(&self) -> Option<&Row> {
        if self.edge_index < self.edges.len() {
            self.edges[self.edge_index].1.get(self.row_index)
        } else {
            None
        }
    }

    fn into_value(self: Box<Self>) -> Option<Row> {
        None
    }

    fn reset(&mut self) {
        self.edge_index = 0;
        self.row_index = 0;
        self.state = if self.edges.is_empty() {
            IteratorState::Finished
        } else {
            IteratorState::Ready
        };
    }

    fn size(&self) -> usize {
        self.edges.iter().map(|(_, rows)| rows.len()).sum()
    }

    fn column_names(&self) -> &[String] {
        &self.column_names
    }

    fn check_memory(&self) -> bool {
        self.check_memory
    }

    fn set_check_memory(&mut self, check: bool) {
        self.check_memory = check;
    }
}

/// 空迭代器
#[derive(Debug)]
pub struct EmptyIter;

impl ResultIterator for EmptyIter {
    fn kind(&self) -> IteratorKind {
        IteratorKind::Default
    }

    fn state(&self) -> IteratorState {
        IteratorState::Finished
    }

    fn valid(&self) -> bool {
        false
    }

    fn next(&mut self) {}

    fn value(&self) -> Option<&Row> {
        None
    }

    fn into_value(self: Box<Self>) -> Option<Row> {
        None
    }

    fn reset(&mut self) {}

    fn size(&self) -> usize {
        0
    }

    fn column_names(&self) -> &[String] {
        &[]
    }

    fn check_memory(&self) -> bool {
        false
    }

    fn set_check_memory(&mut self, _check: bool) {}
}

/// 迭代器工厂
pub struct IteratorFactory;

impl IteratorFactory {
    pub fn create(kind: IteratorKind, data: Vec<Row>, columns: Vec<String>) -> Box<dyn ResultIterator> {
        match kind {
            IteratorKind::Sequential | IteratorKind::Default => {
                Box::new(SequentialIter::new(data, columns))
            }
            IteratorKind::GetNeighbors => {
                Box::new(GetNeighborsIter::new(Vec::new(), columns))
            }
            _ => Box::new(EmptyIter),
        }
    }

    pub fn from_value(value: Value) -> Option<Box<dyn ResultIterator>> {
        match value {
            Value::DataSet(dataset) => Some(Box::new(SequentialIter::new(
                dataset.rows,
                dataset.columns,
            ))),
            _ => None,
        }
    }
}
```

#### 2.3.3 更新 ExecutionResponse

修改 `src/query/context/execution/query_execution.rs`:

```rust
use super::iterator::{ResultIterator, IteratorKind, IteratorState, SequentialIter};

/// 执行响应
#[derive(Debug, Clone)]
pub struct ExecutionResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub iterator: Option<Box<dyn ResultIterator>>,
    pub columns: Vec<String>,
    pub state: ResponseState,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseState {
    UnExecuted,
    PartialSuccess,
    Success,
}

impl ExecutionResponse {
    pub fn new(success: bool) -> Self {
        Self {
            success,
            data: None,
            iterator: None,
            columns: Vec::new(),
            state: if success { ResponseState::Success } else { ResponseState::UnExecuted },
            error_code: None,
            error_message: None,
            execution_time_ms: 0,
        }
    }

    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data.clone());
        self.iterator = SequentialIter::from_value(data);
        self.success = true;
        self.state = ResponseState::Success;
        self
    }

    pub fn with_error(mut self, code: i32, message: String) -> Self {
        self.error_code = Some(code);
        self.error_message = Some(message);
        self.success = false;
        self.state = ResponseState::UnExecuted;
        self
    }

    pub fn with_partial_success(mut self, message: String) -> Self {
        self.error_message = Some(message);
        self.success = true;
        self.state = ResponseState::PartialSuccess;
        self
    }

    pub fn has_iterator(&self) -> bool {
        self.iterator.is_some()
    }

    pub fn iterator(&self) -> Option<&Box<dyn ResultIterator>> {
        self.iterator.as_ref()
    }

    pub fn iterator_mut(&mut self) -> Option<&mut Box<dyn ResultIterator>> {
        self.iterator.as_mut()
    }
}
```

#### 2.3.4 测试用例

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequential_iter() {
        let rows = vec![
            Row { values: vec![Value::Int64(1), Value::String("a".to_string())] },
            Row { values: vec![Value::Int64(2), Value::String("b".to_string())] },
        ];
        let mut iter = SequentialIter::new(rows, vec!["id".to_string(), "name".to_string()]);

        assert_eq!(iter.kind(), IteratorKind::Sequential);
        assert_eq!(iter.size(), 2);
        assert!(iter.valid());

        if let Some(row) = iter.value() {
            assert_eq!(row.values[0], Value::Int64(1));
        }

        iter.next();
        assert!(iter.valid());

        iter.next();
        assert!(!iter.valid());
        assert_eq!(iter.state(), IteratorState::Finished);
    }

    #[test]
    fn test_empty_iter() {
        let mut iter = EmptyIter;
        assert!(!iter.valid());
        assert_eq!(iter.size(), 0);
        assert_eq!(iter.kind(), IteratorKind::Default);
    }

    #[test]
    fn test_execution_response_with_iterator() {
        let value = Value::DataSet(DataSet {
            columns: vec!["col1".to_string()],
            rows: vec![
                Row { values: vec![Value::Int64(1)] },
                Row { values: vec![Value::Int64(2)] },
            ],
        });

        let response = ExecutionResponse::new(false).with_data(value);
        assert!(response.success);
        assert!(response.has_iterator());
        assert_eq!(response.iterator().unwrap().size(), 2);
    }
}
```

---

## 三、中优先级修改详细方案

### 3.1 修改 4：拆分 QueryContext 职责

#### 3.1.1 修改文件

- `src/query/context/execution/query_execution.rs`

#### 3.1.2 修改方案

```rust
/// 核心查询上下文（最小集）
#[derive(Debug, Clone)]
pub struct CoreQueryContext {
    pub vctx: ValidationContext,
    pub ectx: QueryExecutionContext,
    pub plan: Option<ExecutionPlan>,
}

/// 组件访问器（只读引用）
#[derive(Debug, Clone)]
pub struct QueryComponents {
    pub schema_manager: Arc<dyn SchemaManager>,
    pub index_manager: Arc<dyn IndexManager>,
    pub storage_client: Arc<dyn StorageClient>,
    pub meta_client: Arc<dyn MetaClient>,
    pub charset_info: Option<Box<CharsetInfo>>,
}

/// 请求绑定的查询上下文
#[derive(Debug)]
pub struct RequestBoundContext {
    pub rctx: Arc<RequestContext>,
    pub core: CoreQueryContext,
    pub components: QueryComponents,
    pub sym_table: SymbolTable,
    pub obj_pool: ObjectPool<String>,
    pub id_gen: IdGenerator,
    pub killed: bool,
}

impl RequestBoundContext {
    pub fn new(
        rctx: Arc<RequestContext>,
        core: CoreQueryContext,
        components: QueryComponents,
    ) -> Self {
        Self {
            rctx,
            core,
            components,
            sym_table: SymbolTable::new(),
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            killed: false,
        }
    }

    pub fn kill(&mut self) {
        self.killed = true;
    }

    pub fn is_killed(&self) -> bool {
        self.killed
    }
}

/// 简化的 QueryContext（向后兼容）
#[derive(Debug, Clone)]
pub struct QueryContext {
    rctx: Option<Arc<RequestContext>>,
    vctx: ValidationContext,
    ectx: QueryExecutionContext,
    plan: Option<Box<ExecutionPlan>>,
    schema_manager: Option<Arc<dyn SchemaManager>>,
    index_manager: Option<Arc<dyn IndexManager>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    meta_client: Option<Arc<dyn MetaClient>>,
    charset_info: Option<Box<CharsetInfo>>,
    obj_pool: ObjectPool<String>,
    id_gen: IdGenerator,
    sym_table: SymbolTable,
    killed: bool,
}

impl QueryContext {
    pub fn new() -> Self {
        Self {
            rctx: None,
            vctx: ValidationContext::new(),
            ectx: QueryExecutionContext::new(),
            plan: None,
            schema_manager: None,
            index_manager: None,
            storage_client: None,
            meta_client: None,
            charset_info: None,
            obj_pool: ObjectPool::new(1000),
            id_gen: IdGenerator::new(0),
            sym_table: SymbolTable::new(),
            killed: false,
        }
    }

    /// 转换为 RequestBoundContext
    pub fn into_request_bound(
        self,
        rctx: Arc<RequestContext>,
        components: QueryComponents,
    ) -> RequestBoundContext {
        RequestBoundContext::new(
            rctx,
            CoreQueryContext {
                vctx: self.vctx,
                ectx: self.ectx,
                plan: self.plan.map(|p| *p),
            },
            components,
        )
    }

    /// 从 RequestBoundContext 创建
    pub fn from_request_bound(rctx: Arc<RequestContext>, bound: RequestBoundContext) -> Self {
        Self {
            rctx: Some(rctx),
            vctx: bound.core.vctx,
            ectx: bound.core.ectx,
            plan: bound.core.plan.map(Box::new),
            schema_manager: Some(bound.components.schema_manager),
            index_manager: Some(bound.components.index_manager),
            storage_client: Some(bound.components.storage_client),
            meta_client: Some(bound.components.meta_client),
            charset_info: bound.components.charset_info,
            obj_pool: bound.obj_pool,
            id_gen: bound.id_gen,
            sym_table: bound.sym_table,
            killed: bound.killed,
        }
    }

    // 保留原有接口以保持兼容性
    pub fn set_rctx(&mut self, rctx: Arc<RequestContext>) {
        self.rctx = Some(rctx);
    }

    pub fn rctx(&self) -> Option<&RequestContext> {
        self.rctx.as_deref()
    }

    pub fn vctx(&self) -> &ValidationContext {
        &self.vctx
    }

    pub fn vctx_mut(&mut self) -> &mut ValidationContext {
        &mut self.vctx
    }

    pub fn ectx(&self) -> &QueryExecutionContext {
        &self.ectx
    }

    pub fn ectx_mut(&mut self) -> &mut QueryExecutionContext {
        &mut self.ectx
    }

    pub fn set_schema_manager(&mut self, sm: Arc<dyn SchemaManager>) {
        self.schema_manager = Some(sm);
    }

    pub fn set_index_manager(&mut self, im: Arc<dyn IndexManager>) {
        self.index_manager = Some(im);
    }

    pub fn set_storage_client(&mut self, storage: Arc<dyn StorageClient>) {
        self.storage_client = Some(storage);
    }

    pub fn set_meta_client(&mut self, meta_client: Arc<dyn MetaClient>) {
        self.meta_client = Some(meta_client);
    }

    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.schema_manager.as_ref()
    }

    pub fn index_manager(&self) -> Option<&Arc<dyn IndexManager>> {
        self.index_manager.as_ref()
    }

    pub fn get_storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client.as_ref()
    }

    pub fn sym_table(&self) -> &SymbolTable {
        &self.sym_table
    }

    pub fn sym_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.sym_table
    }

    pub fn obj_pool(&self) -> &ObjectPool<String> {
        &self.obj_pool
    }

    pub fn obj_pool_mut(&mut self) -> &mut ObjectPool<String> {
        &mut self.obj_pool
    }

    pub fn gen_id(&self) -> i64 {
        self.id_gen.id()
    }

    pub fn mark_killed(&mut self) {
        self.killed = true;
    }

    pub fn is_killed(&self) -> bool {
        self.killed
    }
}
```

---

### 3.2 修改 5：添加 ExpressionContext

#### 3.2.1 新建文件

- `src/query/context/expression_context.rs`

#### 3.2.2 修改方案

```rust
// src/query/context/expression_context.rs

use crate::core::{Value, Row};
use std::collections::HashMap;

/// 表达式求值上下文
#[derive(Debug, Clone)]
pub struct ExpressionContext {
    variables: HashMap<String, Value>,
    inner_variables: HashMap<String, Value>,
    iter: Option<Row>,
}

impl ExpressionContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            inner_variables: HashMap::new(),
            iter: None,
        }
    }

    pub fn set_var(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    pub fn get_var(&self, name: &str) -> Option<&Value> {
        self.variables.get(name)
    }

    pub fn set_inner_var(&mut self, name: String, value: Value) {
        self.inner_variables.insert(name, value);
    }

    pub fn get_inner_var(&self, name: &str) -> Option<&Value> {
        self.inner_variables.get(name)
    }

    pub fn set_iter(&mut self, row: Row) {
        self.iter = Some(row);
    }

    pub fn clear_iter(&mut self) {
        self.iter = None;
    }

    pub fn iter(&self) -> Option<&Row> {
        self.iter.as_ref()
    }

    pub fn iter_mut(&mut self) -> Option<&mut Row> {
        self.iter.as_mut()
    }

    pub fn get_input_prop(&self, prop: &str) -> Option<&Value> {
        self.iter.as_ref().and_then(|row| {
            row.values.iter().find(|v| match v {
                Value::String(s) if s == prop => true,
                _ => false,
            })
        })
    }

    pub fn get_column(&self, index: usize) -> Option<&Value> {
        self.iter.as_ref().and_then(|row| row.values.get(index))
    }

    pub fn get_var_prop(&self, var: &str, prop: &str) -> Option<&Value> {
        self.variables.get(var).and_then(|value| match value {
            Value::Vertex(v) => v.props.get(prop),
            Value::Edge(e) => e.props.get(prop),
            Value::DataSet(ds) => {
                // 从数据集查找列
                let col_idx = ds.columns.iter().position(|c| c == prop)?;
                ds.rows.first()?.values.get(col_idx)
            }
            _ => None,
        })
    }
}
```

---

### 3.3 修改 6：重构 SchemaManager 接口

#### 3.3.1 修改文件

- `src/query/context/managers/schema_manager.rs`

#### 3.3.2 修改方案

```rust
// 核心读取接口
pub trait SchemaReader: Send + Sync + std::fmt::Debug {
    fn get_schema(&self, name: &str) -> Option<Schema>;
    fn list_schemas(&self) -> Vec<String>;
    fn has_schema(&self, name: &str) -> bool;
    fn get_tag(&self, space_id: i32, tag_id: i32) -> Option<TagDefWithId>;
    fn get_edge_type(&self, space_id: i32, edge_type_id: i32) -> Option<EdgeTypeDefWithId>;
    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDefWithId>>;
    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDefWithId>>;
}

// 写入接口
pub trait SchemaWriter: Send + Sync + std::fmt::Debug {
    fn create_tag(
        &self,
        space_id: i32,
        tag_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;

    fn drop_tag(&self, space_id: i32, tag_id: i32) -> ManagerResult<()>;
    fn alter_tag(&self, space_id: i32, tag_name: &str, alter_cmd: AlterTagCommand) -> ManagerResult<()>;

    fn create_edge_type(
        &self,
        space_id: i32,
        edge_type_name: &str,
        fields: Vec<FieldDef>,
    ) -> ManagerResult<i32>;

    fn drop_edge_type(&self, space_id: i32, edge_type_id: i32) -> ManagerResult<()>;
    fn alter_edge_type(&self, space_id: i32, edge_type_name: &str, alter_cmd: AlterEdgeCommand) -> ManagerResult<()>;
}

// 版本控制接口
pub trait SchemaVersionControl: Send + Sync + std::fmt::Debug {
    fn create_version(&self, space_id: i32, comment: Option<String>) -> ManagerResult<i32>;
    fn get_version(&self, space_id: i32, version: i32) -> Option<SchemaVersion>;
    fn get_latest_version(&self, space_id: i32) -> Option<i32>;
    fn rollback(&self, space_id: i32, version: i32) -> ManagerResult<()>;
}

// 完整接口（兼容旧代码）
pub trait SchemaManager: SchemaReader + SchemaWriter + SchemaVersionControl {
    fn load_from_disk(&self) -> ManagerResult<()>;
    fn save_to_disk(&self) -> ManagerResult<()>;
}
```

---

## 四、低优先级修改

### 4.1 修改 7：优化并发模型

在单节点场景下，移除不必要的 `Arc<RwLock>`：

```rust
// 对于 RequestContext
pub struct RequestContext {
    request_params: RefCell<RequestParams>,  // 使用 RefCell
    response: RefCell<Response>,
    status: Cell<RequestStatus>,  // 简单状态用 Cell
    // ...
}

// 对于 SymbolTable（单线程模式）
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}
```

### 4.2 修改 8：增强生成器作用域管理

```rust
pub struct AnonVarGenerator {
    stacks: Vec<Vec<u64>>,
    current_scope: usize,
}

impl AnonVarGenerator {
    pub fn enter_scope(&mut self) {
        self.stacks.push(vec![0]);
    }

    pub fn exit_scope(&mut self) {
        self.stacks.pop();
    }
}
```

---

## 五、修改顺序建议

### 第一阶段：核心修改

1. **DataType 枚举** - 类型系统基础
2. **QueryError 错误处理** - 错误处理基础
3. **ResultIterator 迭代器** - 执行引擎基础

### 第二阶段：组件拆分

4. **QueryContext 拆分** - 架构重构
5. **ExpressionContext** - 表达式支持
6. **SchemaManager 重构** - 接口优化

### 第三阶段：优化

7. **并发模型优化**
8. **生成器增强**

---

## 六、兼容性说明

所有修改将保持向后兼容：

1. **DataType**: 添加 `from_str` 方法，旧代码仍可使用字符串
2. **QueryError**: 添加 `From` 实现，旧代码无需修改
3. **QueryContext**: 保留原有接口，新增 `into_request_bound` 方法
4. **SchemaManager**: 添加组合 trait，旧实现仍可用

---

## 七、测试计划

### 7.1 单元测试

每个修改必须包含完整的单元测试，测试覆盖率目标：
- DataType: 100%
- QueryError: 100%
- ResultIterator: 100%
- ExpressionContext: 100%

### 7.2 集成测试

修改完成后需通过：
- 现有查询测试
- 性能基准测试
- 错误处理测试

---

## 八、风险与缓解

| 风险 | 缓解措施 |
|------|---------|
| 修改范围过大 | 分阶段实施，每阶段独立测试 |
| 破坏现有功能 | 保持向后兼容，逐步废弃旧接口 |
| 性能下降 | 性能基准测试监控，保留优化选项 |
| 测试覆盖不足 | 补充测试用例，确保 80%+ 覆盖率 |

---

**文档版本**: 1.0  
**创建时间**: 2024  
**最后更新**: 2024
