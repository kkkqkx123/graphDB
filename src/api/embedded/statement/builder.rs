//! 预编译语句构建器模块
//!
//! 提供构建器模式来创建预编译语句

use crate::api::core::{CoreError, CoreResult, QueryApi};
use crate::api::embedded::statement::config::StatementConfig;
use crate::api::embedded::statement::statement::PreparedStatement;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 预编译语句构建器
pub struct PreparedStatementBuilder<S: StorageClient + 'static> {
    query_api: Arc<Mutex<QueryApi<S>>>,
    query: Option<String>,
    space_id: Option<u64>,
    config: StatementConfig,
}

impl<S: StorageClient + Clone + 'static> PreparedStatementBuilder<S> {
    /// 创建新的构建器
    #[allow(dead_code)]
    pub(crate) fn new(query_api: Arc<Mutex<QueryApi<S>>>) -> Self {
        Self {
            query_api,
            query: None,
            space_id: None,
            config: StatementConfig::default(),
        }
    }

    /// 设置查询
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// 设置空间 ID
    pub fn space_id(mut self, space_id: u64) -> Self {
        self.space_id = Some(space_id);
        self
    }

    /// 设置配置
    pub fn config(mut self, config: StatementConfig) -> Self {
        self.config = config;
        self
    }

    /// 构建预编译语句
    pub fn build(self) -> CoreResult<PreparedStatement<S>> {
        let query = self
            .query
            .ok_or_else(|| CoreError::InvalidParameter("查询字符串不能为空".to_string()))?;

        PreparedStatement::with_config(self.query_api, query, self.space_id, self.config)
    }
}
