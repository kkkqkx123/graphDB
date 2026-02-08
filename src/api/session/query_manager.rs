//! 查询管理器
//!
//! 负责跟踪和管理正在运行的查询。

use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    pub fn duration(&self) -> i64 {
        match self.duration_ms {
            Some(d) => d,
            None => {
                self.start_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::from_secs(0))
                    .as_millis() as i64
            }
        }
    }

    pub fn mark_finished(&mut self) {
        self.status = QueryStatus::Finished;
        self.duration_ms = Some(self.duration());
    }

    pub fn mark_failed(&mut self) {
        self.status = QueryStatus::Failed;
        self.duration_ms = Some(self.duration());
    }

    pub fn mark_killed(&mut self) {
        self.status = QueryStatus::Killed;
        self.duration_ms = Some(self.duration());
    }
}

/// 查询管理器
#[derive(Debug)]
pub struct QueryManager {
    queries: Arc<Mutex<HashMap<i64, QueryInfo>>>,
    next_query_id: Arc<Mutex<i64>>,
}

impl QueryManager {
    pub fn new() -> Self {
        Self {
            queries: Arc::new(Mutex::new(HashMap::new())),
            next_query_id: Arc::new(Mutex::new(1)),
        }
    }

    /// 注册新查询
    pub fn register_query(
        &self,
        session_id: i64,
        user_name: String,
        space_name: Option<String>,
        query_text: String,
    ) -> i64 {
        let query_id = {
            let mut next_id = self.next_query_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let query_info = QueryInfo::new(
            query_id,
            session_id,
            user_name,
            space_name,
            query_text,
        );

        let mut queries = self.queries.lock().unwrap();
        queries.insert(query_id, query_info);

        info!("Registered query {} for session {}", query_id, session_id);
        query_id
    }

    /// 获取查询信息
    pub fn get_query(&self, query_id: i64) -> Option<QueryInfo> {
        let queries = self.queries.lock().unwrap();
        queries.get(&query_id).cloned()
    }

    /// 获取所有查询
    pub fn get_all_queries(&self) -> Vec<QueryInfo> {
        let queries = self.queries.lock().unwrap();
        queries.values().cloned().collect()
    }

    /// 获取指定会话的所有查询
    pub fn get_session_queries(&self, session_id: i64) -> Vec<QueryInfo> {
        let queries = self.queries.lock().unwrap();
        queries
            .values()
            .filter(|q| q.session_id == session_id)
            .cloned()
            .collect()
    }

    /// 获取指定用户的所有查询
    pub fn get_user_queries(&self, user_name: &str) -> Vec<QueryInfo> {
        let queries = self.queries.lock().unwrap();
        queries
            .values()
            .filter(|q| q.user_name == user_name)
            .cloned()
            .collect()
    }

    /// 获取正在运行的查询
    pub fn get_running_queries(&self) -> Vec<QueryInfo> {
        let queries = self.queries.lock().unwrap();
        queries
            .values()
            .filter(|q| q.status == QueryStatus::Running)
            .cloned()
            .collect()
    }

    /// 标记查询为完成
    pub fn mark_query_finished(&self, query_id: i64) {
        let mut queries = self.queries.lock().unwrap();
        if let Some(query) = queries.get_mut(&query_id) {
            query.mark_finished();
            info!("Query {} marked as finished", query_id);
        }
    }

    /// 标记查询为失败
    pub fn mark_query_failed(&self, query_id: i64) {
        let mut queries = self.queries.lock().unwrap();
        if let Some(query) = queries.get_mut(&query_id) {
            query.mark_failed();
            info!("Query {} marked as failed", query_id);
        }
    }

    /// 终止查询
    pub fn kill_query(&self, query_id: i64) -> bool {
        let mut queries = self.queries.lock().unwrap();
        if let Some(query) = queries.get_mut(&query_id) {
            if query.status == QueryStatus::Running {
                query.mark_killed();
                info!("Query {} killed", query_id);
                true
            } else {
                warn!("Cannot kill query {}: status is {:?}", query_id, query.status);
                false
            }
        } else {
            warn!("Query {} not found", query_id);
            false
        }
    }

    /// 清理已完成的查询
    pub fn cleanup_finished_queries(&self, max_age: Duration) {
        let mut queries = self.queries.lock().unwrap();
        let now = SystemTime::now();
        let to_remove: Vec<i64> = queries
            .iter()
            .filter(|(_, q)| {
                if q.status == QueryStatus::Running {
                    return false;
                }
                match now.duration_since(q.start_time) {
                    Ok(duration) => duration > max_age,
                    Err(_) => true,
                }
            })
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            queries.remove(&id);
            info!("Cleaned up finished query {}", id);
        }
    }

    /// 获取查询统计信息
    pub fn get_query_stats(&self) -> QueryStats {
        let queries = self.queries.lock().unwrap();
        let total = queries.len();
        let running = queries.values().filter(|q| q.status == QueryStatus::Running).count();
        let finished = queries.values().filter(|q| q.status == QueryStatus::Finished).count();
        let failed = queries.values().filter(|q| q.status == QueryStatus::Failed).count();
        let killed = queries.values().filter(|q| q.status == QueryStatus::Killed).count();

        QueryStats {
            total_queries: total,
            running_queries: running,
            finished_queries: finished,
            failed_queries: failed,
            killed_queries: killed,
        }
    }
}

impl Default for QueryManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询统计信息
#[derive(Debug, Clone, Default)]
pub struct QueryStats {
    pub total_queries: usize,
    pub running_queries: usize,
    pub finished_queries: usize,
    pub failed_queries: usize,
    pub killed_queries: usize,
}

use std::sync::OnceLock;

/// 全局查询管理器实例
pub static GLOBAL_QUERY_MANAGER: OnceLock<Arc<QueryManager>> = OnceLock::new();
