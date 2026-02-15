use log::{info, warn};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::api::service::permission_manager::RoleType;
use crate::core::error::{SessionError, QueryResult};

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub name: String,
    pub id: i64,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: i64,
    pub user_name: String,
    pub space_name: Option<String>,
    pub graph_addr: Option<String>,
    pub timezone: Option<i32>,
}

/// ClientSession saves those information, including who created it, executed queries,
/// space role, etc. One user corresponds to one ClientSession.
#[derive(Debug)]
pub struct ClientSession {
    session: Arc<RwLock<Session>>,
    space: Arc<RwLock<Option<SpaceInfo>>>,
    roles: Arc<RwLock<HashMap<i64, RoleType>>>,
    idle_start_time: Arc<RwLock<Instant>>,
    contexts: Arc<RwLock<HashMap<u32, String>>>, // Represents queries running in this session
}

impl ClientSession {
    pub fn new(session: Session) -> Arc<Self> {
        Arc::new(Self {
            session: Arc::new(RwLock::new(session)),
            space: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            idle_start_time: Arc::new(RwLock::new(Instant::now())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn id(&self) -> i64 {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .session_id
    }

    pub fn space(&self) -> Option<SpaceInfo> {
        self.space.read().expect("Space lock was poisoned").clone()
    }

    pub fn set_space(&self, space: SpaceInfo) {
        *self.space.write().expect("Space lock was poisoned") = Some(space);
    }

    pub fn space_name(&self) -> Option<String> {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .space_name
            .clone()
    }

    pub fn user(&self) -> String {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .user_name
            .clone()
    }

    pub fn roles(&self) -> HashMap<i64, RoleType> {
        self.roles.read().expect("Roles lock was poisoned").clone()
    }

    pub fn role_with_space(&self, space: i64) -> Option<RoleType> {
        self.roles
            .read()
            .expect("Roles lock was poisoned")
            .get(&space)
            .cloned()
    }

    /// 检查用户是否是God角色（全局超级管理员）
    /// 只要用户在任意Space拥有God角色，就是God用户
    pub fn is_god(&self) -> bool {
        self.roles
            .read()
            .expect("Roles lock was poisoned")
            .values()
            .any(|role| *role == RoleType::God)
    }

    /// 检查用户是否是Admin角色（Space管理员）
    /// Admin或God都被视为管理员
    pub fn is_admin(&self) -> bool {
        self.roles
            .read()
            .expect("Roles lock was poisoned")
            .values()
            .any(|role| *role == RoleType::Admin || *role == RoleType::God)
    }

    pub fn set_role(&self, space: i64, role: RoleType) {
        self.roles
            .write()
            .expect("Roles lock was poisoned")
            .insert(space, role);
    }

    pub fn idle_seconds(&self) -> u64 {
        self.idle_start_time
            .read()
            .expect("Idle start time lock was poisoned")
            .elapsed()
            .as_secs()
    }

    pub fn charge(&self) {
        *self
            .idle_start_time
            .write()
            .expect("Idle start time lock was poisoned") = Instant::now();
    }

    pub fn timezone(&self) -> Option<i32> {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .timezone
    }

    pub fn set_timezone(&self, timezone: i32) {
        self.session
            .write()
            .expect("Session lock was poisoned")
            .timezone = Some(timezone);
    }

    pub fn graph_addr(&self) -> Option<String> {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .graph_addr
            .clone()
    }

    pub fn update_graph_addr(&self, host_addr: String) {
        self.session
            .write()
            .expect("Session lock was poisoned")
            .graph_addr = Some(host_addr);
    }

    pub fn get_session(&self) -> Session {
        self.session
            .read()
            .expect("Session lock was poisoned")
            .clone()
    }

    pub fn update_space_name(&self, space_name: String) {
        self.session
            .write()
            .expect("Session lock was poisoned")
            .space_name = Some(space_name);
    }

    pub fn add_query(&self, ep_id: u32, query_context: String) {
        info!("Adding query {} to session {}", ep_id, self.id());
        self.contexts
            .write()
            .expect("Contexts lock was poisoned")
            .insert(ep_id, query_context);
    }

    pub fn delete_query(&self, ep_id: u32) {
        info!("Removing query {} from session {}", ep_id, self.id());
        self.contexts
            .write()
            .expect("Contexts lock was poisoned")
            .remove(&ep_id);
    }

    pub fn find_query(&self, ep_id: u32) -> bool {
        self.contexts
            .read()
            .expect("Contexts lock was poisoned")
            .contains_key(&ep_id)
    }

    pub fn mark_query_killed(&self, ep_id: u32) {
        // In a real implementation, this would mark query as killed in context
        // For now, we'll just remove it
        self.contexts
            .write()
            .expect("Contexts lock was poisoned")
            .remove(&ep_id);
    }

    pub fn mark_all_queries_killed(&self) {
        let query_count = self.active_queries_count();
        info!("Killing all {} queries in session {}", query_count, self.id());
        self.contexts
            .write()
            .expect("Contexts lock was poisoned")
            .clear();
    }

    /// 获取当前活动的查询数量
    pub fn active_queries_count(&self) -> usize {
        self.contexts
            .read()
            .expect("Contexts lock was poisoned")
            .len()
    }

    /// 终止指定查询（KILL QUERY）
    /// 
    /// # 参数
    /// * `query_id` - 要终止的查询ID
    /// 
    /// # 返回
    /// * `Ok(())` - 成功终止查询
    /// * `Err(QueryError)` - 终止失败的具体原因
    pub fn kill_query(&self, query_id: u32) -> QueryResult<()> {
        info!("Attempting to kill query {} in session {}", query_id, self.id());
        
        // 检查查询是否存在
        if !self.find_query(query_id) {
            warn!("Query {} not found in session {}", query_id, self.id());
            return Err(SessionError::QueryNotFound(query_id));
        }
        
        // 标记查询为已终止
        self.mark_query_killed(query_id);
        
        info!("Successfully killed query {} in session {}", query_id, self.id());
        Ok(())
    }

    /// 批量终止多个查询
    pub fn kill_multiple_queries(&self, query_ids: &[u32]) -> Vec<QueryResult<()>> {
        query_ids.iter().map(|&query_id| {
            self.kill_query(query_id)
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_session_creation() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        assert_eq!(client_session.id(), 123);
        assert_eq!(client_session.user(), "testuser");
        assert_eq!(client_session.roles().len(), 0);
        assert!(!client_session.is_admin());
    }

    #[test]
    fn test_session_space_management() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        let space = SpaceInfo {
            name: "test_space".to_string(),
            id: 1,
        };

        client_session.set_space(space.clone());
        assert_eq!(
            client_session.space().expect("Space should exist").name,
            "test_space"
        );
        assert_eq!(client_session.space().expect("Space should exist").id, 1);
    }

    #[test]
    fn test_session_roles() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        client_session.set_role(1, RoleType::Admin);
        assert!(matches!(
            client_session.role_with_space(1),
            Some(RoleType::Admin)
        ));
        assert!(client_session.role_with_space(2).is_none());

        // Test is_admin function
        assert!(client_session.is_admin());

        // Add User role and verify is_admin returns true (has Admin role)
        client_session.set_role(2, RoleType::User);
        assert!(client_session.is_admin()); // Still has Admin role from space 1
    }

    #[test]
    fn test_session_queries() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Add a query
        client_session.add_query(1, "SELECT * FROM users".to_string());
        assert!(client_session.find_query(1));
        assert!(!client_session.find_query(2));

        // Remove a query
        client_session.delete_query(1);
        assert!(!client_session.find_query(1));

        // Mark all queries killed
        client_session.add_query(1, "SELECT * FROM users".to_string());
        client_session.add_query(2, "INSERT INTO users VALUES (...)".to_string());
        client_session.mark_all_queries_killed();
        assert!(!client_session.find_query(1));
        assert!(!client_session.find_query(2));
    }

    #[test]
    fn test_session_idle_time() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Check initial idle time is a valid u64 value
        let initial_idle = client_session.idle_seconds();

        // Charge session (reset idle time)
        client_session.charge();
        assert!(client_session.idle_seconds() <= initial_idle); // Should be close to 0 after charge
    }

    #[test]
    fn test_kill_query() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Add some queries
        client_session.add_query(1, "SELECT * FROM users".to_string());
        client_session.add_query(2, "INSERT INTO users VALUES (...)".to_string());
        assert_eq!(client_session.active_queries_count(), 2);

        // Kill a specific query
        let result = client_session.kill_query(1);
        assert!(result.is_ok());
        assert!(!client_session.find_query(1));
        assert!(client_session.find_query(2));
        assert_eq!(client_session.active_queries_count(), 1);

        // Try to kill non-existent query
        let result = client_session.kill_query(999);
        assert!(matches!(result, Err(SessionError::QueryNotFound(999))));
    }

    #[test]
    fn test_kill_multiple_queries() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Add multiple queries
        client_session.add_query(1, "SELECT * FROM users".to_string());
        client_session.add_query(2, "INSERT INTO users VALUES (...)".to_string());
        client_session.add_query(3, "UPDATE users SET ...".to_string());
        assert_eq!(client_session.active_queries_count(), 3);

        // Kill multiple queries
        let results = client_session.kill_multiple_queries(&[1, 3]);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());

        assert!(!client_session.find_query(1));
        assert!(client_session.find_query(2));
        assert!(!client_session.find_query(3));
        assert_eq!(client_session.active_queries_count(), 1);
    }
}
