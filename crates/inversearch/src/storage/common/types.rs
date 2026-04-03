//! 存储模块共享类型
//!
//! 定义所有存储实现共享的通用数据结构和类型

use serde::{Deserialize, Serialize};

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// 文件存储数据格式
///
/// 用于文件存储和缓存存储的序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageData {
    pub version: String,
    pub timestamp: String,
    pub data: std::collections::HashMap<String, Vec<crate::r#type::DocId>>,
    pub context_data: std::collections::HashMap<
        String,
        std::collections::HashMap<String, Vec<crate::r#type::DocId>>,
    >,
    pub documents: std::collections::HashMap<crate::r#type::DocId, String>,
}
