//! 存储客户端接口 - 定义存储层访问的基本操作

use crate::core::Value;
use std::result::Result;

/// 存储操作类型
#[derive(Debug, Clone)]
pub enum StorageOperation {
    Read {
        table: String,
        key: String,
    },
    Write {
        table: String,
        key: String,
        value: Value,
    },
    Delete {
        table: String,
        key: String,
    },
    Scan {
        table: String,
        prefix: String,
    },
}

/// 存储响应
#[derive(Debug, Clone)]
pub struct StorageResponse {
    pub success: bool,
    pub data: Option<Value>,
    pub error_message: Option<String>,
}

/// 存储客户端接口 - 定义存储层访问的基本操作
pub trait StorageClient: Send + Sync + std::fmt::Debug {
    /// 执行存储操作
    fn execute(&self, operation: StorageOperation) -> Result<StorageResponse, String>;
    /// 检查连接状态
    fn is_connected(&self) -> bool;
}
