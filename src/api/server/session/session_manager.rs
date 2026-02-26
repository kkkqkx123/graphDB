use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time;
use dashmap::DashMap;

use super::network_session::{ClientSession, Session};
use crate::core::error::{SessionError, SessionResult};

pub const DEFAULT_MAX_ALLOWED_CONNECTIONS: usize = 100; // 默认最大连接数（单节点场景）
pub const DEFAULT_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10分钟

/// 全局会话ID计数器，用于生成唯一的会话ID
static SESSION_ID_COUNTER: AtomicU64 = AtomicU64::new(0);

/// 会话信息，用于展示会话列表
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: i64,
    pub user_name: String,
    pub space_name: Option<String>,
    pub graph_addr: Option<String>,
    pub create_time: SystemTime,
    pub last_access_time: SystemTime,
    pub active_queries: usize,
    pub timezone: Option<i32>,
}

impl SessionInfo {
    pub fn from_client_session(session: &ClientSession, create_time: SystemTime) -> Self {
        Self {
            session_id: session.id(),
            user_name: session.user(),
            space_name: session.space_name(),
            graph_addr: session.graph_addr(),
            create_time,
            last_access_time: SystemTime::now() - Duration::from_secs(session.idle_seconds()),
            active_queries: session.active_queries_count(),
            timezone: session.timezone(),
        }
    }
}

#[derive(Debug)]
pub struct GraphSessionManager {
    // 使用 DashMap 实现真正的并发访问，无需显式加锁
    sessions: Arc<DashMap<i64, Arc<ClientSession>>>,
    active_sessions: Arc<DashMap<i64, Instant>>, // session_id -> last_activity_time
    // 读多写少，使用 tokio::RwLock
    session_create_times: Arc<RwLock<HashMap<i64, SystemTime>>>, // session_id -> create_time
    host_addr: String,
    max_connections: usize,
    session_idle_timeout: Duration,
    /// 后台清理任务是否正在运行
    cleanup_task_running: Arc<AtomicBool>,
}

impl GraphSessionManager {
    /// 创建新的会话管理器
    ///
    /// 注意：此构造函数不会自动启动后台清理任务，
    /// 需要显式调用 `start_cleanup_task()` 来启动
    pub fn new(host_addr: String, max_connections: usize, session_idle_timeout: Duration) -> Arc<Self> {
        Arc::new(Self {
            sessions: Arc::new(DashMap::new()),
            active_sessions: Arc::new(DashMap::new()),
            session_create_times: Arc::new(RwLock::new(HashMap::new())),
            host_addr,
            max_connections,
            session_idle_timeout,
            cleanup_task_running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// 启动后台会话清理任务
    ///
    /// 如果任务已经在运行，此方法不会重复启动
    pub fn start_cleanup_task(self: &Arc<Self>) {
        if self.cleanup_task_running.swap(true, Ordering::SeqCst) {
            info!("Session cleanup task is already running");
            return;
        }

        info!("Starting session cleanup task");
        let manager_clone = Arc::clone(self);
        tokio::spawn(async move {
            manager_clone.background_reclamation_task().await;
        });
    }

    /// 停止后台会话清理任务
    ///
    /// 设置停止标志，后台任务将在下一次循环时退出
    pub fn stop_cleanup_task(&self) {
        info!("Stopping session cleanup task");
        self.cleanup_task_running.store(false, Ordering::SeqCst);
    }

    /// 检查后台清理任务是否正在运行
    pub fn is_cleanup_task_running(&self) -> bool {
        self.cleanup_task_running.load(Ordering::SeqCst)
    }

    /// Creates a new session
    pub async fn create_session(
        &self,
        user_name: String,
        _client_ip: String,
    ) -> Result<Arc<ClientSession>, String> {
        info!("Creating new session for user: {}", user_name);
        
        // Check if we're out of connections
        if self.is_out_of_connections().await {
            warn!("Failed to create session for user {}: maximum connections exceeded", user_name);
            return Err("Exceeded maximum allowed connections".to_string());
        }

        // Generate a new session ID
        let session_id = self.generate_session_id();
        info!("Generated session ID: {} for user: {}", session_id, user_name);

        let session = Session {
            session_id,
            user_name: user_name.clone(),
            space_name: None,
            graph_addr: Some(self.host_addr.clone()),
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Add to sessions and active sessions
        let create_time = SystemTime::now();
        
        // DashMap 无需显式加锁，真正的并发插入
        self.sessions.insert(session_id, Arc::clone(&client_session));
        self.active_sessions.insert(session_id, Instant::now());
        
        // 写锁保护创建时间
        {
            let mut create_times = self.session_create_times.write().await;
            create_times.insert(session_id, create_time);
        }

        info!("Successfully created session ID: {} for user: {}", session_id, user_name);
        Ok(client_session)
    }

    /// Finds an existing session
    pub fn find_session(&self, session_id: i64) -> Option<Arc<ClientSession>> {
        // DashMap 支持真正的并发读，无需加锁
        self.sessions.get(&session_id).map(|entry| entry.clone())
    }

    /// Finds an existing session only from local cache
    pub fn find_session_from_cache(&self, session_id: i64) -> Option<Arc<ClientSession>> {
        self.find_session(session_id)
    }

    /// Removes a session from local cache
    pub async fn remove_session(&self, session_id: i64) {
        info!("Removing session ID: {}", session_id);
        
        // DashMap 无需显式加锁
        self.sessions.remove(&session_id);
        self.active_sessions.remove(&session_id);
        
        // 写锁保护创建时间
        {
            let mut create_times = self.session_create_times.write().await;
            create_times.remove(&session_id);
        }
        
        info!("Successfully removed session ID: {}", session_id);
    }

    /// Gets all sessions from the local cache
    pub fn get_sessions_from_local_cache(&self) -> Vec<Session> {
        // DashMap 支持迭代器，无需加锁
        self.sessions
            .iter()
            .map(|entry| entry.value().get_session())
            .collect()
    }

    /// 获取会话列表信息，用于SHOW SESSIONS
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        // 读锁获取创建时间
        let create_times = self.session_create_times.read().await;
        
        // DashMap 迭代无需加锁
        self.sessions
            .iter()
            .filter_map(|entry| {
                let session_id = entry.key();
                let client_session = entry.value();
                create_times.get(session_id).map(|&create_time| {
                    SessionInfo::from_client_session(client_session, create_time)
                })
            })
            .collect()
    }

    /// 获取指定会话的详细信息
    pub async fn get_session_info(&self, session_id: i64) -> Option<SessionInfo> {
        // DashMap 读无需加锁
        let client_session = self.sessions.get(&session_id)?;
        
        // 读锁获取创建时间
        let create_times = self.session_create_times.read().await;
        create_times.get(&session_id).map(|&create_time| {
            SessionInfo::from_client_session(&client_session, create_time)
        })
    }

    /// 终止指定会话（KILL SESSION）
    /// 
    /// # 参数
    /// * `session_id` - 要终止的会话ID
    /// * `current_user` - 执行终止操作的用户名
    /// * `is_admin` - 当前用户是否为Admin角色
    /// 
    /// # 返回
    /// * `Ok(())` - 成功终止会话
    /// * `Err(SessionError)` - 终止失败的具体原因
    pub async fn kill_session(&self, session_id: i64, current_user: &str, is_admin: bool) -> SessionResult<()> {
        info!("Attempting to kill session ID: {} by user: {} (is_admin: {})", session_id, current_user, is_admin);
        
        // 查找目标会话
        let target_session = self.find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;
        
        let target_user = target_session.user();
        
        // 权限检查：只能终止自己的会话，或者有Admin权限
        if !is_admin && target_user != current_user {
            warn!("User {} attempted to kill session {} without permission (target user: {})", 
                  current_user, session_id, target_user);
            return Err(SessionError::InsufficientPermission);
        }
        
        info!("Killing session {} (user: {}, active queries: {})", 
              session_id, target_user, target_session.active_queries_count());
        
        // 终止会话中的所有查询
        target_session.mark_all_queries_killed();
        
        // 从管理器中移除会话
        self.remove_session(session_id).await;
        
        info!("Successfully killed session ID: {} by user: {}", session_id, current_user);
        Ok(())
    }

    /// 批量终止多个会话
    pub async fn kill_multiple_sessions(&self, session_ids: &[i64], current_user: &str, is_admin: bool) -> Vec<SessionResult<()>> {
        let mut results = Vec::with_capacity(session_ids.len());
        for &session_id in session_ids {
            results.push(self.kill_session(session_id, current_user, is_admin).await);
        }
        results
    }

    /// Whether exceeds the max allowed connections
    pub async fn is_out_of_connections(&self) -> bool {
        // DashMap 的 len() 是 O(1) 操作，无需加锁
        self.active_sessions.len() >= self.max_connections
    }

    /// Generate a new unique session ID
    /// 
    /// 使用组合策略生成唯一会话ID：
    /// - 高48位：当前时间戳（毫秒）
    /// - 低16位：自增计数器
    /// 确保在同一毫秒内生成的ID也是唯一的
    fn generate_session_id(&self) -> i64 {
        let timestamp_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_millis() as u64;
        
        let counter = SESSION_ID_COUNTER.fetch_add(1, Ordering::SeqCst) & 0xFFFF;
        
        // 组合时间戳和计数器
        let session_id = ((timestamp_millis & 0xFFFFFFFFFFFF0000) | counter) as i64;
        
        // 确保生成的ID为正数且不为0
        if session_id <= 0 {
            // 如果生成的ID无效，使用时间戳的哈希值
            ((timestamp_millis.wrapping_mul(0x9E3779B97F4A7C15)) & 0x7FFFFFFFFFFFFFFF) as i64
        } else {
            session_id
        }
    }

    /// 后台任务：定期清理过期会话
    ///
    /// 每30秒检查一次，清理超过空闲超时的会话
    /// 可以通过 `stop_cleanup_task()` 方法停止
    async fn background_reclamation_task(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            // 检查是否应该停止
            if !self.cleanup_task_running.load(Ordering::SeqCst) {
                info!("Session cleanup task is stopping");
                break;
            }

            self.reclaim_expired_sessions().await;
        }

        info!("Session cleanup task has stopped");
    }

    /// Reclaims expired sessions
    async fn reclaim_expired_sessions(&self) {
        // DashMap 支持迭代无需加锁
        let expired_sessions: Vec<i64> = self.active_sessions
            .iter()
            .filter(|entry| entry.value().elapsed() > self.session_idle_timeout)
            .map(|entry| *entry.key())
            .collect();

        if !expired_sessions.is_empty() {
            info!("Found {} expired sessions to reclaim", expired_sessions.len());
        }

        // Remove expired sessions
        for session_id in expired_sessions {
            info!("Reclaiming expired session ID: {}", session_id);
            self.remove_session(session_id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_session_manager() -> Arc<GraphSessionManager> {
        GraphSessionManager::new(
            "127.0.0.1:9669".to_string(),
            DEFAULT_MAX_ALLOWED_CONNECTIONS,
            DEFAULT_SESSION_IDLE_TIMEOUT,
        )
    }

    #[tokio::test]
    async fn test_session_manager_creation() {
        let session_manager = create_test_session_manager();

        assert_eq!(session_manager.host_addr, "127.0.0.1:9669");
        assert_eq!(session_manager.get_sessions_from_local_cache().len(), 0);
        assert!(!session_manager.is_cleanup_task_running());
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let session_manager = create_test_session_manager();

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .await
            .expect("Failed to create session");

        assert_eq!(session.user(), "testuser");
        assert!(!session_manager.is_out_of_connections().await);

        let found_session = session_manager
            .find_session(session.id())
            .expect("Failed to find session");
        assert_eq!(found_session.user(), "testuser");

        // Test find non-existent session
        assert!(session_manager.find_session(999999).is_none());
    }

    #[tokio::test]
    async fn test_remove_session() {
        let session_manager = create_test_session_manager();

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .await
            .expect("Failed to create session");

        assert!(session_manager.find_session(session.id()).is_some());

        session_manager.remove_session(session.id()).await;
        assert!(session_manager.find_session(session.id()).is_none());
    }

    #[tokio::test]
    async fn test_max_connections() {
        let session_manager = GraphSessionManager::new(
            "127.0.0.1:9669".to_string(),
            5,
            DEFAULT_SESSION_IDLE_TIMEOUT,
        );

        assert!(!session_manager.is_out_of_connections().await);

        for i in 0..5 {
            let _ = session_manager.create_session(
                format!("user{}", i),
                "127.0.0.1".to_string()
            ).await;
        }

        assert!(session_manager.is_out_of_connections().await);

        // 尝试创建第6个会话应该失败
        let result = session_manager.create_session(
            "user6".to_string(),
            "127.0.0.1".to_string()
        ).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_kill_session() {
        let session_manager = create_test_session_manager();

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .await
            .expect("Failed to create session");

        let session_id = session.id();

        // 普通用户尝试终止自己的会话 - 应该成功
        let result = session_manager.kill_session(session_id, "testuser", false).await;
        assert!(result.is_ok());
        assert!(session_manager.find_session(session_id).is_none());

        // 创建新会话测试权限检查
        let session2 = session_manager
            .create_session("user2".to_string(), "127.0.0.1".to_string())
            .await
            .expect("Failed to create session");

        // 普通用户尝试终止其他用户的会话 - 应该失败
        let result = session_manager.kill_session(session2.id(), "otheruser", false).await;
        assert!(result.is_err());

        // Admin 终止其他用户的会话 - 应该成功
        let result = session_manager.kill_session(session2.id(), "admin", true).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let session_manager = create_test_session_manager();

        // 创建多个会话
        for i in 0..3 {
            let _ = session_manager.create_session(
                format!("user{}", i),
                "127.0.0.1".to_string()
            ).await;
        }

        let sessions = session_manager.list_sessions().await;
        assert_eq!(sessions.len(), 3);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use tokio::task;

        let session_manager = create_test_session_manager();
        let mut handles = vec![];

        // 并发创建会话
        for i in 0..10 {
            let manager = Arc::clone(&session_manager);
            let handle = task::spawn(async move {
                manager.create_session(
                    format!("user{}", i),
                    "127.0.0.1".to_string()
                ).await
            });
            handles.push(handle);
        }

        // 等待所有任务完成
        for handle in handles {
            let _ = handle.await.unwrap();
        }

        // 验证所有会话都创建成功
        assert_eq!(session_manager.get_sessions_from_local_cache().len(), 10);
    }
}
