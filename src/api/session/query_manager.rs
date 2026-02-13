//! 查询管理器
//!
//! 负责跟踪和管理正在运行的查询。

use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

    pub fn duration(&self) -> ManagerResult<i64> {
        match self.duration_ms {
            Some(d) => Ok(d),
            None => {
                self.start_time
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis() as i64)
                    .map_err(|_| ManagerError::Other("时间计算错误".to_string()))
            }
        }
    }

    pub fn mark_finished(&mut self) -> ManagerResult<()> {
        self.status = QueryStatus::Finished;
        self.duration_ms = Some(self.duration()?);
        Ok(())
    }

    pub fn mark_failed(&mut self) -> ManagerResult<()> {
        self.status = QueryStatus::Failed;
        self.duration_ms = Some(self.duration()?);
        Ok(())
    }

    pub fn mark_killed(&mut self) -> ManagerResult<()> {
        self.status = QueryStatus::Killed;
        self.duration_ms = Some(self.duration()?);
        Ok(())
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
    ) -> ManagerResult<i64> {
        let query_id = {
            let mut next_id = self.next_query_id.lock()
                .map_err(|e| ManagerError::Other(format!("获取查询 ID 锁失败: {}", e)))?;
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

        let mut queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        queries.insert(query_id, query_info);

        info!("Registered query {} for session {}", query_id, session_id);
        Ok(query_id)
    }

    /// 获取查询信息
    pub fn get_query(&self, query_id: i64) -> ManagerResult<QueryInfo> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        queries.get(&query_id)
            .cloned()
            .ok_or_else(|| ManagerError::NotFound(format!("查询 {} 不存在", query_id)))
    }

    /// 获取所有查询
    pub fn get_all_queries(&self) -> ManagerResult<Vec<QueryInfo>> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        Ok(queries.values().cloned().collect())
    }

    /// 获取指定会话的所有查询
    pub fn get_session_queries(&self, session_id: i64) -> ManagerResult<Vec<QueryInfo>> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        Ok(queries
            .values()
            .filter(|q| q.session_id == session_id)
            .cloned()
            .collect())
    }

    /// 获取指定用户的所有查询
    pub fn get_user_queries(&self, user_name: &str) -> ManagerResult<Vec<QueryInfo>> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        Ok(queries
            .values()
            .filter(|q| q.user_name == user_name)
            .cloned()
            .collect())
    }

    /// 获取正在运行的查询
    pub fn get_running_queries(&self) -> ManagerResult<Vec<QueryInfo>> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        Ok(queries
            .values()
            .filter(|q| q.status == QueryStatus::Running)
            .cloned()
            .collect())
    }

    /// 标记查询为完成
    pub fn mark_query_finished(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        let query = queries.get_mut(&query_id)
            .ok_or_else(|| ManagerError::NotFound(format!("查询 {} 不存在", query_id)))?;
        query.mark_finished()?;
        info!("Query {} marked as finished", query_id);
        Ok(())
    }

    /// 标记查询为失败
    pub fn mark_query_failed(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        let query = queries.get_mut(&query_id)
            .ok_or_else(|| ManagerError::NotFound(format!("查询 {} 不存在", query_id)))?;
        query.mark_failed()?;
        info!("Query {} marked as failed", query_id);
        Ok(())
    }

    /// 终止查询
    pub fn kill_query(&self, query_id: i64) -> ManagerResult<()> {
        let mut queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        let query = queries.get_mut(&query_id)
            .ok_or_else(|| ManagerError::NotFound(format!("查询 {} 不存在", query_id)))?;
        
        if query.status == QueryStatus::Running {
            query.mark_killed()?;
            info!("Query {} killed", query_id);
            Ok(())
        } else {
            warn!("Cannot kill query {}: status is {:?}", query_id, query.status);
            Err(ManagerError::InvalidInput(format!("查询 {} 的状态 {:?} 不允许终止", query_id, query.status)))
        }
    }

    /// 清理已完成的查询
    pub fn cleanup_finished_queries(&self, max_age: Duration) -> ManagerResult<()> {
        let mut queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
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
        Ok(())
    }

    /// 获取查询统计信息
    pub fn get_query_stats(&self) -> ManagerResult<QueryStats> {
        let queries = self.queries.lock()
            .map_err(|e| ManagerError::Other(format!("获取查询锁失败: {}", e)))?;
        let total = queries.len();
        let running = queries.values().filter(|q| q.status == QueryStatus::Running).count();
        let finished = queries.values().filter(|q| q.status == QueryStatus::Finished).count();
        let failed = queries.values().filter(|q| q.status == QueryStatus::Failed).count();
        let killed = queries.values().filter(|q| q.status == QueryStatus::Killed).count();

        Ok(QueryStats {
            total_queries: total,
            running_queries: running,
            finished_queries: finished,
            failed_queries: failed,
            killed_queries: killed,
        })
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
