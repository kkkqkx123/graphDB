//! 查询管理器
//!
//! 负责跟踪和管理正在运行的查询。

use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::sync::OnceLock;

use crate::core::error::{ManagerError, ManagerResult};

/// 查询状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryStatus {
    Running,
    Finished,
    Failed,
    Killed,
}

/// 查询信息
#[derive(Debug, Clone)]
pub struct QueryInfo {
    pub query_id: i64,
    pub session_id: i64,
    pub user_name: String,
    pub space_name: Option<String>,
    pub query_text: String,
    pub status: QueryStatus,
    pub start_time: SystemTime,
    pub duration_ms: Option<i64>,
    pub execution_plan: Option<String>,
}

impl QueryInfo {
    pub fn new(
        query_id: i64,
        session_id: i64,
        user_name: String,
        space_name: Option<String>,
        query_text: String,
    ) -> Self {
        Self {
            query_id,
            session_id,
            user_name,
            space_name,
            query_text,
            status: QueryStatus::Running,
            start_time: SystemTime::now(),
            duration_ms: None,
            execution_plan: None,
        }
    }

    pub fn finish(&mut self) {
        self.status = QueryStatus::Finished;
        self.duration_ms = Some(
            SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default()
                .as_millis() as i64,
        );
    }

    pub fn fail(&mut self) {
        self.status = QueryStatus::Failed;
        self.duration_ms = Some(
            SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default()
                .as_millis() as i64,
        );
    }

    pub fn kill(&mut self) {
        self.status = QueryStatus::Killed;
        self.duration_ms = Some(
            SystemTime::now()
                .duration_since(self.start_time)
                .unwrap_or_default()
                .as_millis() as i64,
        );
    }
}

/// 查询统计
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub total_queries: u64,
    pub running_queries: u64,
    pub finished_queries: u64,
    pub failed_queries: u64,
    pub killed_queries: u64,
    pub avg_duration_ms: i64,
}

/// 查询管理器
pub struct QueryManager {
    queries: Mutex<HashMap<i64, QueryInfo>>,
    next_query_id: Mutex<i64>,
}

impl QueryManager {
    pub fn new() -> Self {
        Self {
            queries: Mutex::new(HashMap::new()),
            next_query_id: Mutex::new(1),
        }
    }

    /// 生成新的查询ID
    fn generate_query_id(&self) -> i64 {
        let mut id = self.next_query_id.lock();
        let query_id = *id;
        *id += 1;
        query_id
    }

    /// 注册新查询
    pub fn register_query(
        &self,
        session_id: i64,
        user_name: String,
        space_name: Option<String>,
        query_text: String,
    ) -> i64 {
        let query_id = self.generate_query_id();
        let query_info = QueryInfo::new(
            query_id,
            session_id,
            user_name,
            space_name,
            query_text.clone(),
        );

        let mut queries = self.queries.lock();
        queries.insert(query_id, query_info);

        info!("Query registered: id={}, session_id={}, query={}", query_id, session_id, query_text);

        query_id
    }

    /// 完成查询
    pub fn finish_query(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock();
        if let Some(query) = queries.get_mut(&query_id) {
            query.finish();
            info!("Query finished: id={}, duration={}ms", query_id, query.duration_ms.unwrap_or(0));
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("Query {} not found", query_id)))
        }
    }

    /// 标记查询失败
    pub fn fail_query(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock();
        if let Some(query) = queries.get_mut(&query_id) {
            query.fail();
            warn!("Query failed: id={}, duration={}ms", query_id, query.duration_ms.unwrap_or(0));
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("Query {} not found", query_id)))
        }
    }

    /// 终止查询
    pub fn kill_query(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock();
        if let Some(query) = queries.get_mut(&query_id) {
            query.kill();
            warn!("Query killed: id={}", query_id);
            Ok(())
        } else {
            Err(ManagerError::NotFound(format!("Query {} not found", query_id)))
        }
    }

    /// 获取查询信息
    pub fn get_query(&self, query_id: i64) -> Option<QueryInfo> {
        let queries = self.queries.lock();
        queries.get(&query_id).cloned()
    }

    /// 获取所有查询
    pub fn get_all_queries(&self) -> Vec<QueryInfo> {
        let queries = self.queries.lock();
        queries.values().cloned().collect()
    }

    /// 获取正在运行的查询
    pub fn get_running_queries(&self) -> Vec<QueryInfo> {
        let queries = self.queries.lock();
        queries
            .values()
            .filter(|q| q.status == QueryStatus::Running)
            .cloned()
            .collect()
    }

    /// 获取查询统计
    pub fn get_stats(&self) -> QueryStats {
        let queries = self.queries.lock();
        let total = queries.len() as u64;
        let running = queries.values().filter(|q| q.status == QueryStatus::Running).count() as u64;
        let finished = queries.values().filter(|q| q.status == QueryStatus::Finished).count() as u64;
        let failed = queries.values().filter(|q| q.status == QueryStatus::Failed).count() as u64;
        let killed = queries.values().filter(|q| q.status == QueryStatus::Killed).count() as u64;

        let total_duration: i64 = queries
            .values()
            .filter_map(|q| q.duration_ms)
            .sum();

        let avg_duration = if total > 0 {
            total_duration / total as i64
        } else {
            0
        };

        QueryStats {
            total_queries: total,
            running_queries: running,
            finished_queries: finished,
            failed_queries: failed,
            killed_queries: killed,
            avg_duration_ms: avg_duration,
        }
    }

    /// 清理已完成的查询（保留最近N个）
    pub fn cleanup_finished_queries(&self, keep_count: usize) {
        let mut queries = self.queries.lock();
        let mut finished_queries: Vec<_> = queries
            .iter()
            .filter(|(_, q)| q.status != QueryStatus::Running)
            .map(|(id, _)| *id)
            .collect();

        // 按开始时间排序，保留最近的
        finished_queries.sort_by_key(|id| {
            queries.get(id).map(|q| q.start_time).unwrap_or(UNIX_EPOCH)
        });

        let to_remove = finished_queries.len().saturating_sub(keep_count);
        for id in finished_queries.into_iter().take(to_remove) {
            queries.remove(&id);
        }
    }
}

impl Default for QueryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局查询管理器
pub static GLOBAL_QUERY_MANAGER: OnceLock<Arc<QueryManager>> = OnceLock::new();

/// 初始化全局查询管理器
pub fn init_global_query_manager() -> Arc<QueryManager> {
    let manager = Arc::new(QueryManager::new());
    GLOBAL_QUERY_MANAGER.set(manager.clone()).ok();
    manager
}

/// 获取全局查询管理器
pub fn get_global_query_manager() -> Option<Arc<QueryManager>> {
    GLOBAL_QUERY_MANAGER.get().cloned()
}
