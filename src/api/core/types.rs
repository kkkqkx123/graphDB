//! API 核心层类型定义
//!
//! 与传输层无关的业务类型

use crate::core::Value;
use std::collections::HashMap;

/// 查询执行上下文
#[derive(Debug, Clone)]
pub struct QueryContext {
    pub space_id: Option<u64>,
    pub auto_commit: bool,
    pub transaction_id: Option<u64>,
    pub parameters: Option<HashMap<String, Value>>,
}

impl Default for QueryContext {
    fn default() -> Self {
        Self {
            space_id: None,
            auto_commit: true,
            transaction_id: None,
            parameters: None,
        }
    }
}

/// 查询结果
#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Row>,
    pub metadata: ExecutionMetadata,
}

/// 结果行
#[derive(Debug)]
pub struct Row {
    pub values: HashMap<String, Value>,
}

impl Row {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: HashMap::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, key: String, value: Value) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
}

/// 执行元数据
#[derive(Debug, Clone)]
pub struct ExecutionMetadata {
    pub execution_time_ms: u64,
    pub rows_scanned: u64,
    pub rows_returned: u64,
    pub cache_hit: bool,
}

impl Default for ExecutionMetadata {
    fn default() -> Self {
        Self {
            execution_time_ms: 0,
            rows_scanned: 0,
            rows_returned: 0,
            cache_hit: false,
        }
    }
}

/// 事务句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransactionHandle(pub u64);

/// 保存点 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SavepointId(pub u64);

/// Schema 属性定义
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub name: String,
    pub data_type: crate::core::DataType,
    pub nullable: bool,
    pub default_value: Option<Value>,
    pub comment: Option<String>,
}

/// 索引目标类型
#[derive(Debug, Clone)]
pub enum IndexTarget {
    Tag { name: String },
    Edge { name: String },
}

/// 空间配置
#[derive(Debug, Clone)]
pub struct SpaceConfig {
    pub partition_num: i32,
    pub replica_factor: i32,
    pub vid_type: crate::core::DataType,
    pub comment: Option<String>,
}

impl Default for SpaceConfig {
    fn default() -> Self {
        Self {
            partition_num: 100,
            replica_factor: 1,
            vid_type: crate::core::DataType::String,
            comment: None,
        }
    }
}
