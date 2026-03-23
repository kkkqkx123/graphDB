//! 预编译语句管理器

use crate::api::core::{CoreError, CoreResult};
use crate::api::server::statement::types::*;
use crate::storage::StorageClient;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

/// 预编译语句管理器
pub struct StatementManager<S: StorageClient + Clone + 'static> {
    /// 存储所有预编译语句
    statements: Arc<DashMap<StatementId, StatementInfo>>,
    /// 存储客户端（预留，用于后续执行语句时使用）
    _storage: Arc<S>,
}

impl<S: StorageClient + Clone + 'static> StatementManager<S> {
    /// 创建新的语句管理器
    pub fn new(storage: Arc<S>) -> Self {
        Self {
            statements: Arc::new(DashMap::new()),
            _storage: storage,
        }
    }

    /// 创建预编译语句
    pub fn create_statement(&self, query: String, space_id: u64) -> CoreResult<StatementInfo> {
        // TODO: 解析查询语句，提取参数
        // 目前先简单实现，假设查询中包含 $param 形式的参数
        let parameters = Self::extract_parameters(&query);

        let statement_id = Uuid::new_v4().to_string();
        let info = StatementInfo::new(statement_id.clone(), query, space_id, parameters);

        self.statements.insert(statement_id.clone(), info.clone());
        Ok(info)
    }

    /// 获取预编译语句
    pub fn get_statement(&self, statement_id: &str) -> Option<StatementInfo> {
        self.statements.get(statement_id).map(|s| s.clone())
    }

    /// 执行预编译语句
    pub fn execute_statement(
        &self,
        statement_id: &str,
        _parameters: &std::collections::HashMap<String, serde_json::Value>,
    ) -> CoreResult<ExecuteStatementResponse> {
        let mut info = self.statements.get_mut(statement_id).ok_or_else(|| {
            CoreError::InvalidParameter(format!("预编译语句不存在: {}", statement_id))
        })?;

        // TODO: 实际执行查询
        // 目前返回模拟结果
        let start = std::time::Instant::now();

        // 模拟执行
        let execution_time_ms = start.elapsed().as_millis() as u64;
        info.record_execution(execution_time_ms);

        Ok(ExecuteStatementResponse {
            data: None, // TODO: 返回实际查询数据
            metadata: StatementMetadata {
                execution_time_ms,
                rows_returned: 0,
            },
        })
    }

    /// 批量执行预编译语句
    pub fn batch_execute_statement(
        &self,
        statement_id: &str,
        batch_parameters: Vec<std::collections::HashMap<String, serde_json::Value>>,
    ) -> CoreResult<BatchExecuteStatementResponse> {
        let mut results = Vec::new();
        let mut success = 0;
        let mut failed = 0;

        for parameters in batch_parameters {
            match self.execute_statement(statement_id, &parameters) {
                Ok(response) => {
                    success += 1;
                    results.push(response);
                }
                Err(_) => {
                    failed += 1;
                    // TODO: 记录错误
                }
            }
        }

        let total = results.len();
        Ok(BatchExecuteStatementResponse {
            results,
            summary: BatchSummary {
                total,
                success,
                failed,
            },
        })
    }

    /// 删除预编译语句
    pub fn remove_statement(&self, statement_id: &str) -> CoreResult<()> {
        self.statements.remove(statement_id).ok_or_else(|| {
            CoreError::InvalidParameter(format!("预编译语句不存在: {}", statement_id))
        })?;
        Ok(())
    }

    /// 提取查询参数
    fn extract_parameters(query: &str) -> Vec<String> {
        // 简单实现：查找 $param 形式的参数
        let mut parameters = Vec::new();
        let mut chars = query.chars().peekable();

        while let Some(c) = chars.next() {
            if c == '$' {
                let mut param_name = String::new();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_alphanumeric() || next_c == '_' {
                        param_name.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if !param_name.is_empty() && !parameters.contains(&param_name) {
                    parameters.push(param_name);
                }
            }
        }

        parameters
    }

    /// 获取所有语句ID
    pub fn list_statements(&self) -> Vec<StatementId> {
        self.statements
            .iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// 清理长时间未使用的语句
    pub fn cleanup_old_statements(&self, max_age_hours: i64) {
        let now = chrono::Utc::now();
        let max_age = chrono::Duration::hours(max_age_hours);

        let mut to_remove = Vec::new();
        for entry in self.statements.iter() {
            if now - entry.value().last_used_at > max_age {
                to_remove.push(entry.key().clone());
            }
        }

        for id in to_remove {
            let _ = self.statements.remove(&id);
        }
    }
}
