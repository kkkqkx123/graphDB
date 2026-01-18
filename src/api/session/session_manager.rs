use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::time;

use super::client_session::{ClientSession, Session};
use crate::core::error::{SessionError, SessionResult};

pub const DEFAULT_MAX_ALLOWED_CONNECTIONS: usize = 1000; // 默认最大连接数
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
    sessions: Arc<Mutex<HashMap<i64, Arc<ClientSession>>>>,
    active_sessions: Arc<Mutex<HashMap<i64, Instant>>>, // session_id -> last_activity_time
    session_create_times: Arc<Mutex<HashMap<i64, SystemTime>>>, // session_id -> create_time
    host_addr: String,
    max_connections: usize,
    session_idle_timeout: Duration,
}

impl GraphSessionManager {
    pub fn new(host_addr: String, max_connections: usize, session_idle_timeout: Duration) -> Arc<Self> {
        let manager = Arc::new(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            session_create_times: Arc::new(Mutex::new(HashMap::new())),
            host_addr,
            max_connections,
            session_idle_timeout,
        });

        let manager_clone = Arc::clone(&manager);
        tokio::spawn(async move {
            manager_clone.background_reclamation_task().await;
        });

        manager
    }

    /// Creates a new session
    pub fn create_session(
        &self,
        user_name: String,
        _client_ip: String,
    ) -> Result<Arc<ClientSession>, String> {
        info!("Creating new session for user: {}", user_name);
        
        // Check if we're out of connections
        if self.is_out_of_connections() {
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
        {
            let mut sessions = self.sessions.lock().expect("Sessions lock was poisoned");
            let mut active_sessions = self
                .active_sessions
                .lock()
                .expect("Active sessions lock was poisoned");
            let mut session_create_times = self
                .session_create_times
                .lock()
                .expect("Session create times lock was poisoned");

            sessions.insert(session_id, Arc::clone(&client_session));
            active_sessions.insert(session_id, Instant::now());
            session_create_times.insert(session_id, create_time);
        }

        info!("Successfully created session ID: {} for user: {}", session_id, user_name);
        Ok(client_session)
    }

    /// Finds an existing session
    pub fn find_session(&self, session_id: i64) -> Option<Arc<ClientSession>> {
        let sessions = self.sessions.lock().expect("Sessions lock was poisoned");
        sessions.get(&session_id).cloned()
    }

    /// Finds an existing session only from local cache
    pub fn find_session_from_cache(&self, session_id: i64) -> Option<Arc<ClientSession>> {
        self.find_session(session_id)
    }

    /// Removes a session from local cache
    pub fn remove_session(&self, session_id: i64) {
        info!("Removing session ID: {}", session_id);
        {
            let mut sessions = self.sessions.lock().expect("Sessions lock was poisoned");
            let mut active_sessions = self
                .active_sessions
                .lock()
                .expect("Active sessions lock was poisoned");
            let mut session_create_times = self
                .session_create_times
                .lock()
                .expect("Session create times lock was poisoned");

            sessions.remove(&session_id);
            active_sessions.remove(&session_id);
            session_create_times.remove(&session_id);
        }
        info!("Successfully removed session ID: {}", session_id);
    }

    /// Gets all sessions from the local cache
    pub fn get_sessions_from_local_cache(&self) -> Vec<Session> {
        let sessions = self.sessions.lock().expect("Sessions lock was poisoned");
        sessions
            .values()
            .map(|session| session.get_session())
            .collect()
    }

    /// 获取会话列表信息，用于SHOW SESSIONS
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.lock().expect("Sessions lock was poisoned");
        let create_times = self.session_create_times.lock().expect("Session create times lock was poisoned");
        
        sessions
            .iter()
            .filter_map(|(session_id, client_session)| {
                create_times.get(session_id).map(|&create_time| {
                    SessionInfo::from_client_session(client_session, create_time)
                })
            })
            .collect()
    }

    /// 获取指定会话的详细信息
    pub fn get_session_info(&self, session_id: i64) -> Option<SessionInfo> {
        let sessions = self.sessions.lock().expect("Sessions lock was poisoned");
        let create_times = self.session_create_times.lock().expect("Session create times lock was poisoned");
        
        sessions.get(&session_id).and_then(|client_session| {
            create_times.get(&session_id).map(|&create_time| {
                SessionInfo::from_client_session(client_session, create_time)
            })
        })
    }

    /// 终止指定会话（KILL SESSION）
    /// 
    /// # 参数
    /// * `session_id` - 要终止的会话ID
    /// * `current_user` - 执行终止操作的用户名
    /// * `is_god` - 当前用户是否为God角色
    /// 
    /// # 返回
    /// * `Ok(())` - 成功终止会话
    /// * `Err(SessionError)` - 终止失败的具体原因
    pub fn kill_session(&self, session_id: i64, current_user: &str, is_god: bool) -> SessionResult<()> {
        info!("Attempting to kill session ID: {} by user: {} (is_god: {})", session_id, current_user, is_god);
        
        // 查找目标会话
        let target_session = self.find_session(session_id)
            .ok_or(SessionError::SessionNotFound(session_id))?;
        
        let target_user = target_session.user();
        
        // 权限检查：只能终止自己的会话，或者有God权限
        if !is_god && target_user != current_user {
            warn!("User {} attempted to kill session {} without permission (target user: {})", 
                  current_user, session_id, target_user);
            return Err(SessionError::PermissionDenied);
        }
        
        info!("Killing session {} (user: {}, active queries: {})", 
              session_id, target_user, target_session.active_queries_count());
        
        // 终止会话中的所有查询
        target_session.mark_all_queries_killed();
        
        // 从管理器中移除会话
        self.remove_session(session_id);
        
        info!("Successfully killed session ID: {} by user: {}", session_id, current_user);
        Ok(())
    }

    /// 批量终止多个会话
    pub fn kill_multiple_sessions(&self, session_ids: &[i64], current_user: &str, is_god: bool) -> Vec<SessionResult<()>> {
        session_ids.iter().map(|&session_id| {
            self.kill_session(session_id, current_user, is_god)
        }).collect()
    }

    /// Whether exceeds the max allowed connections
    pub fn is_out_of_connections(&self) -> bool {
        let active_sessions = self
            .active_sessions
            .lock()
            .expect("Active sessions lock was poisoned");
        active_sessions.len() >= self.max_connections
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

    /// Background task to reclaim expired sessions
    async fn background_reclamation_task(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;
            self.reclaim_expired_sessions();
        }
    }

    /// Reclaims expired sessions
    fn reclaim_expired_sessions(&self) {
        let active_sessions = self
            .active_sessions
            .lock()
            .expect("Active sessions lock was poisoned");
        let expired_sessions: Vec<i64> = active_sessions
            .iter()
            .filter(|(_, last_activity)| last_activity.elapsed() > self.session_idle_timeout)
            .map(|(&session_id, _)| session_id)
            .collect();
        drop(active_sessions);

        if !expired_sessions.is_empty() {
            info!("Found {} expired sessions to reclaim", expired_sessions.len());
        }

        // Remove expired sessions
        for session_id in expired_sessions {
            info!("Reclaiming expired session ID: {}", session_id);
            self.remove_session(session_id);
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
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let session_manager = create_test_session_manager();

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session");

        assert_eq!(session.user(), "testuser");
        assert!(!session_manager.is_out_of_connections());

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
            .expect("Failed to create session");

        assert!(session_manager.find_session(session.id()).is_some());

        session_manager.remove_session(session.id());
        assert!(session_manager.find_session(session.id()).is_none());
    }

    #[tokio::test]
    async fn test_max_connections() {
        let session_manager = GraphSessionManager::new(
            "127.0.0.1:9669".to_string(),
            5,
            DEFAULT_SESSION_IDLE_TIMEOUT,
        );

        assert!(!session_manager.is_out_of_connections());

        for i in 0..5 {
            let _ = session_manager.create_session(
                format!("user{}", i),
                "127.0.0.1".to_string()
            );
        }

        assert!(session_manager.is_out_of_connections());
    }

    #[tokio::test]
    async fn test_session_cache_operations() {
        let session_manager = create_test_session_manager();

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session");

        // Test finding from cache
        let cached_session = session_manager
            .find_session_from_cache(session.id())
            .expect("Failed to find cached session");
        assert_eq!(cached_session.user(), "testuser");

        // Test getting all sessions from cache
        let all_sessions = session_manager.get_sessions_from_local_cache();
        assert_eq!(all_sessions.len(), 1);
        assert_eq!(all_sessions[0].session_id, session.id());
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let session_manager = create_test_session_manager();

        // Create multiple sessions
        let session1 = session_manager
            .create_session("user1".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session 1");
        
        let _session2 = session_manager
            .create_session("user2".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session 2");

        // Test list_sessions
        let session_infos = session_manager.list_sessions();
        assert_eq!(session_infos.len(), 2);
        
        // Verify session info content
        let user_names: Vec<String> = session_infos.iter().map(|info| info.user_name.clone()).collect();
        assert!(user_names.contains(&"user1".to_string()));
        assert!(user_names.contains(&"user2".to_string()));
        
        // Test get_session_info for specific session
        let info1 = session_manager.get_session_info(session1.id());
        assert!(info1.is_some());
        assert_eq!(info1.expect("info1 should be Some").user_name, "user1");
        
        let info_nonexistent = session_manager.get_session_info(999999);
        assert!(info_nonexistent.is_none());
    }

    #[tokio::test]
    async fn test_kill_session() {
        let session_manager = create_test_session_manager();

        // Create a session
        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session");
        
        let session_id = session.id();

        // Add some queries to the session
        session.add_query(1, "SELECT * FROM users".to_string());
        session.add_query(2, "INSERT INTO users VALUES (...)".to_string());
        assert_eq!(session.active_queries_count(), 2);

        // Kill the session as the same user (should succeed)
        let result = session_manager.kill_session(session_id, "testuser", false);
        assert!(result.is_ok());
        
        // Verify session is removed
        assert!(session_manager.find_session(session_id).is_none());
        
        // Test killing non-existent session
        let result = session_manager.kill_session(999999, "testuser", false);
        assert!(matches!(result, Err(SessionError::SessionNotFound(_))));
    }

    #[tokio::test]
    async fn test_kill_session_permission_denied() {
        let session_manager = create_test_session_manager();

        // Create a session for user1
        let session = session_manager
            .create_session("user1".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session");
        
        let session_id = session.id();

        // Try to kill the session as user2 (should fail)
        let result = session_manager.kill_session(session_id, "user2", false);
        assert!(matches!(result, Err(SessionError::PermissionDenied)));
        
        // Verify session still exists
        assert!(session_manager.find_session(session_id).is_some());
        
        // But God user should be able to kill it
        let result = session_manager.kill_session(session_id, "goduser", true);
        assert!(result.is_ok());
        assert!(session_manager.find_session(session_id).is_none());
    }
}
