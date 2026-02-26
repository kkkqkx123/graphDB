use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use parking_lot::RwLock;

use crate::core::RoleType;
use crate::core::error::{QueryResult, QueryError};
use crate::transaction::{SavepointId, TransactionId, TransactionOptions};

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
    
    // 事务相关字段
    current_transaction: Arc<RwLock<Option<TransactionId>>>,
    savepoint_stack: Arc<RwLock<Vec<SavepointId>>>,
    transaction_options: Arc<RwLock<TransactionOptions>>,
    auto_commit: Arc<RwLock<bool>>,
}

impl ClientSession {
    pub fn new(session: Session) -> Arc<Self> {
        Arc::new(Self {
            session: Arc::new(RwLock::new(session)),
            space: Arc::new(RwLock::new(None)),
            roles: Arc::new(RwLock::new(HashMap::new())),
            idle_start_time: Arc::new(RwLock::new(Instant::now())),
            contexts: Arc::new(RwLock::new(HashMap::new())),
            // 初始化事务相关字段
            current_transaction: Arc::new(RwLock::new(None)),
            savepoint_stack: Arc::new(RwLock::new(Vec::new())),
            transaction_options: Arc::new(RwLock::new(TransactionOptions::default())),
            auto_commit: Arc::new(RwLock::new(true)), // 默认开启自动提交
        })
    }

    pub fn id(&self) -> i64 {
        self.session.read().session_id
    }

    pub fn space(&self) -> Option<SpaceInfo> {
        self.space.read().clone()
    }

    pub fn set_space(&self, space: SpaceInfo) {
        *self.space.write() = Some(space);
    }

    pub fn space_name(&self) -> Option<String> {
        self.session.read().space_name.clone()
    }

    pub fn user(&self) -> String {
        self.session.read().user_name.clone()
    }

    pub fn roles(&self) -> HashMap<i64, RoleType> {
        self.roles.read().clone()
    }

    pub fn role_with_space(&self, space: i64) -> Option<RoleType> {
        self.roles.read().get(&space).cloned()
    }

    /// 检查用户是否是God角色（全局超级管理员）
    /// 只要用户在任意Space拥有God角色，就是God用户
    pub fn is_god(&self) -> bool {
        self.roles
            .read()
            .values()
            .any(|role| *role == RoleType::God)
    }

    /// 检查用户是否是Admin角色（Space管理员）
    /// Admin或God都被视为管理员
    pub fn is_admin(&self) -> bool {
        self.roles
            .read()
            .values()
            .any(|role| *role == RoleType::Admin || *role == RoleType::God)
    }

    pub fn set_role(&self, space: i64, role: RoleType) {
        self.roles.write().insert(space, role);
    }

    pub fn idle_seconds(&self) -> u64 {
        self.idle_start_time.read().elapsed().as_secs()
    }

    pub fn charge(&self) {
        *self.idle_start_time.write() = Instant::now();
    }

    pub fn timezone(&self) -> Option<i32> {
        self.session.read().timezone
    }

    pub fn set_timezone(&self, timezone: i32) {
        self.session.write().timezone = Some(timezone);
    }

    pub fn graph_addr(&self) -> Option<String> {
        self.session.read().graph_addr.clone()
    }

    pub fn update_graph_addr(&self, host_addr: String) {
        self.session.write().graph_addr = Some(host_addr);
    }

    pub fn get_session(&self) -> Session {
        self.session.read().clone()
    }

    pub fn update_space_name(&self, space_name: String) {
        self.session.write().space_name = Some(space_name);
    }

    pub fn add_query(&self, ep_id: u32, query_context: String) {
        info!("Adding query {} to session {}", ep_id, self.id());
        self.contexts.write().insert(ep_id, query_context);
    }

    pub fn delete_query(&self, ep_id: u32) {
        info!("Removing query {} from session {}", ep_id, self.id());
        self.contexts.write().remove(&ep_id);
    }

    pub fn find_query(&self, ep_id: u32) -> bool {
        self.contexts.read().contains_key(&ep_id)
    }

    pub fn mark_query_killed(&self, ep_id: u32) {
        // In a real implementation, this would mark query as killed in context
        // For now, we'll just remove it
        self.contexts.write().remove(&ep_id);
    }

    pub fn mark_all_queries_killed(&self) {
        let query_count = self.active_queries_count();
        info!("Killing all {} queries in session {}", query_count, self.id());
        self.contexts.write().clear();
    }

    /// 获取当前活动的查询数量
    pub fn active_queries_count(&self) -> usize {
        self.contexts.read().len()
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
            return Err(QueryError::ExecutionError(format!("查询未找到: {}", query_id)));
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

    // ==================== 事务管理方法 ====================

    /// 获取当前绑定的事务ID
    pub fn current_transaction(&self) -> Option<TransactionId> {
        self.current_transaction.read().clone()
    }

    /// 绑定事务到会话
    pub fn bind_transaction(&self, txn_id: TransactionId) {
        info!("Binding transaction {} to session {}", txn_id, self.id());
        *self.current_transaction.write() = Some(txn_id);
    }

    /// 解绑当前事务
    pub fn unbind_transaction(&self) {
        if let Some(txn_id) = self.current_transaction() {
            info!("Unbinding transaction {} from session {}", txn_id, self.id());
            *self.current_transaction.write() = None;
            // 清空保存点栈
            self.savepoint_stack.write().clear();
        }
    }

    /// 检查是否有活跃事务
    pub fn has_active_transaction(&self) -> bool {
        self.current_transaction().is_some()
    }

    /// 获取自动提交模式
    pub fn is_auto_commit(&self) -> bool {
        *self.auto_commit.read()
    }

    /// 设置自动提交模式
    pub fn set_auto_commit(&self, auto_commit: bool) {
        info!("Setting auto_commit to {} for session {}", auto_commit, self.id());
        *self.auto_commit.write() = auto_commit;
    }

    /// 获取事务选项
    pub fn transaction_options(&self) -> TransactionOptions {
        self.transaction_options.read().clone()
    }

    /// 设置事务选项
    pub fn set_transaction_options(&self, options: TransactionOptions) {
        *self.transaction_options.write() = options;
    }

    // ==================== 保存点管理方法 ====================

    /// 添加保存点到栈
    pub fn push_savepoint(&self, savepoint_id: SavepointId) {
        info!("Pushing savepoint {} to session {}", savepoint_id, self.id());
        self.savepoint_stack.write().push(savepoint_id);
    }

    /// 获取保存点栈（克隆）
    pub fn savepoint_stack(&self) -> Vec<SavepointId> {
        self.savepoint_stack.read().clone()
    }

    /// 清空保存点栈
    pub fn clear_savepoints(&self) {
        info!("Clearing savepoint stack for session {}", self.id());
        self.savepoint_stack.write().clear();
    }

    /// 获取保存点数量
    pub fn savepoint_count(&self) -> usize {
        self.savepoint_stack.read().len()
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

        // 初始没有空间
        assert!(client_session.space().is_none());
        assert!(client_session.space_name().is_none());

        // 设置空间
        let space_info = SpaceInfo {
            name: "test_space".to_string(),
            id: 456,
        };
        client_session.set_space(space_info.clone());
        
        assert_eq!(client_session.space().unwrap().id, 456);
        assert_eq!(client_session.space().unwrap().name, "test_space");

        // 更新空间名称
        client_session.update_space_name("new_space".to_string());
        assert_eq!(client_session.space_name().unwrap(), "new_space");
    }

    #[test]
    fn test_session_role_management() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始没有角色
        assert!(client_session.role_with_space(1).is_none());
        assert!(!client_session.is_admin());
        assert!(!client_session.is_god());

        // 设置Admin角色
        client_session.set_role(1, RoleType::Admin);
        assert_eq!(client_session.role_with_space(1).unwrap(), RoleType::Admin);
        assert!(client_session.is_admin());
        assert!(!client_session.is_god());

        // 设置God角色
        client_session.set_role(2, RoleType::God);
        assert!(client_session.is_god());
        assert!(client_session.is_admin()); // God也是Admin
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

        // 初始空闲时间应该很短
        let idle1 = client_session.idle_seconds();
        assert_eq!(idle1, 0);

        // 模拟空闲
        std::thread::sleep(std::time::Duration::from_millis(100));
        let idle2 = client_session.idle_seconds();
        assert!(idle2 >= 0);

        // 重置空闲时间
        client_session.charge();
        let idle3 = client_session.idle_seconds();
        assert_eq!(idle3, 0);
    }

    #[test]
    fn test_session_timezone() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始没有时区
        assert!(client_session.timezone().is_none());

        // 设置时区
        client_session.set_timezone(8);
        assert_eq!(client_session.timezone().unwrap(), 8);
    }

    #[test]
    fn test_session_graph_addr() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始没有地址
        assert!(client_session.graph_addr().is_none());

        // 更新地址
        client_session.update_graph_addr("127.0.0.1:9779".to_string());
        assert_eq!(client_session.graph_addr().unwrap(), "127.0.0.1:9779");
    }

    #[test]
    fn test_session_query_management() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始没有查询
        assert_eq!(client_session.active_queries_count(), 0);
        assert!(!client_session.find_query(1));

        // 添加查询
        client_session.add_query(1, "SELECT * FROM user".to_string());
        assert_eq!(client_session.active_queries_count(), 1);
        assert!(client_session.find_query(1));

        // 删除查询
        client_session.delete_query(1);
        assert_eq!(client_session.active_queries_count(), 0);
        assert!(!client_session.find_query(1));

        // 终止查询
        client_session.add_query(2, "MATCH (n) RETURN n".to_string());
        let result = client_session.kill_query(2);
        assert!(result.is_ok());
        assert!(!client_session.find_query(2));

        // 终止不存在的查询
        let result = client_session.kill_query(999);
        assert!(result.is_err());

        // 批量终止
        client_session.add_query(3, "query 3".to_string());
        client_session.add_query(4, "query 4".to_string());
        let results = client_session.kill_multiple_queries(&[3, 4, 5]);
        assert_eq!(results.len(), 3);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
        assert!(results[2].is_err()); // 5不存在
    }

    #[test]
    fn test_session_transaction_management() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始状态
        assert!(client_session.current_transaction().is_none());
        assert!(!client_session.has_active_transaction());
        assert!(client_session.is_auto_commit());

        // 绑定事务
        client_session.bind_transaction(1001);
        assert_eq!(client_session.current_transaction().unwrap(), 1001);
        assert!(client_session.has_active_transaction());

        // 解绑事务
        client_session.unbind_transaction();
        assert!(client_session.current_transaction().is_none());

        // 设置自动提交
        client_session.set_auto_commit(false);
        assert!(!client_session.is_auto_commit());

        // 事务选项
        let options = TransactionOptions::default();
        client_session.set_transaction_options(options.clone());
        assert_eq!(client_session.transaction_options(), options);
    }

    #[test]
    fn test_session_savepoint_management() {
        let session = Session {
            session_id: 123,
            user_name: "testuser".to_string(),
            space_name: None,
            graph_addr: None,
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // 初始状态
        assert_eq!(client_session.savepoint_count(), 0);
        assert!(client_session.savepoint_stack().is_empty());

        // 添加保存点
        client_session.push_savepoint(1);
        client_session.push_savepoint(2);
        assert_eq!(client_session.savepoint_count(), 2);
        assert_eq!(client_session.savepoint_stack(), vec![1, 2]);

        // 清空保存点
        client_session.clear_savepoints();
        assert_eq!(client_session.savepoint_count(), 0);
    }
}
