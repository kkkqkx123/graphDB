//! 组件访问器模块
//!
//! 提供对查询所需的各种管理器和客户端组件的访问接口。
//! 将SchemaManager、IndexManager、StorageClient、MetaClient等集中管理。

use crate::query::context::managers::{
    CharsetInfo, IndexManager, MetaClient, SchemaManager, StorageClient,
};
use std::sync::Arc;

/// 组件访问器
///
/// 封装所有外部组件的访问，提供统一的接口。
/// 使用Arc包装以支持跨查询共享。
///
/// # 示例
///
/// ```ignore
/// let components = QueryComponents::new()
///     .with_schema_manager(schema_manager)
///     .with_index_manager(index_manager)
///     .with_storage_client(storage_client);
/// ```
#[derive(Debug, Clone)]
pub struct QueryComponents {
    schema_manager: Option<Arc<dyn SchemaManager>>,
    index_manager: Option<Arc<dyn IndexManager>>,
    storage_client: Option<Arc<dyn StorageClient>>,
    meta_client: Option<Arc<dyn MetaClient>>,
    charset_info: Option<Box<CharsetInfo>>,
}

impl QueryComponents {
    /// 创建新的组件访问器
    pub fn new() -> Self {
        Self {
            schema_manager: None,
            index_manager: None,
            storage_client: None,
            meta_client: None,
            charset_info: None,
        }
    }

    /// 设置Schema管理器
    pub fn with_schema_manager(mut self, schema_manager: Arc<dyn SchemaManager>) -> Self {
        self.schema_manager = Some(schema_manager);
        self
    }

    /// 设置索引管理器
    pub fn with_index_manager(mut self, index_manager: Arc<dyn IndexManager>) -> Self {
        self.index_manager = Some(index_manager);
        self
    }

    /// 设置存储客户端
    pub fn with_storage_client(mut self, storage_client: Arc<dyn StorageClient>) -> Self {
        self.storage_client = Some(storage_client);
        self
    }

    /// 设置元数据客户端
    pub fn with_meta_client(mut self, meta_client: Arc<dyn MetaClient>) -> Self {
        self.meta_client = Some(meta_client);
        self
    }

    /// 设置字符集信息
    pub fn with_charset_info(mut self, charset_info: CharsetInfo) -> Self {
        self.charset_info = Some(Box::new(charset_info));
        self
    }

    /// 获取Schema管理器
    pub fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.schema_manager.as_ref()
    }

    /// 获取索引管理器
    pub fn index_manager(&self) -> Option<&Arc<dyn IndexManager>> {
        self.index_manager.as_ref()
    }

    /// 获取存储客户端
    pub fn storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client.as_ref()
    }

    /// 获取元数据客户端
    pub fn meta_client(&self) -> Option<&Arc<dyn MetaClient>> {
        self.meta_client.as_ref()
    }

    /// 获取字符集信息
    pub fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info.as_ref().map(|c| c.as_ref())
    }

    /// 检查是否所有必需组件都已设置
    pub fn is_complete(&self) -> bool {
        self.schema_manager.is_some()
            && self.index_manager.is_some()
            && self.storage_client.is_some()
            && self.meta_client.is_some()
    }

    /// 获取缺失的必需组件
    pub fn missing_components(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_manager.is_none() {
            missing.push("schema_manager");
        }
        if self.index_manager.is_none() {
            missing.push("index_manager");
        }
        if self.storage_client.is_none() {
            missing.push("storage_client");
        }
        if self.meta_client.is_none() {
            missing.push("meta_client");
        }
        missing
    }
}

impl Default for QueryComponents {
    fn default() -> Self {
        Self::new()
    }
}

/// 组件访问器Trait
///
/// 提供对组件的只读访问接口。
/// 用于需要在不修改组件的情况下访问组件的场景。
pub trait ComponentAccessor {
    fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>>;
    fn index_manager(&self) -> Option<&Arc<dyn IndexManager>>;
    fn storage_client(&self) -> Option<&Arc<dyn StorageClient>>;
    fn meta_client(&self) -> Option<&Arc<dyn MetaClient>>;
    fn charset_info(&self) -> Option<&CharsetInfo>;
}

impl ComponentAccessor for QueryComponents {
    fn schema_manager(&self) -> Option<&Arc<dyn SchemaManager>> {
        self.schema_manager()
    }

    fn index_manager(&self) -> Option<&Arc<dyn IndexManager>> {
        self.index_manager()
    }

    fn storage_client(&self) -> Option<&Arc<dyn StorageClient>> {
        self.storage_client()
    }

    fn meta_client(&self) -> Option<&Arc<dyn MetaClient>> {
        self.meta_client()
    }

    fn charset_info(&self) -> Option<&CharsetInfo> {
        self.charset_info()
    }
}
