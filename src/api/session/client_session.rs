use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct SpaceInfo {
    pub name: String,
    pub id: i64,
    // Additional space description fields would go here
}

#[derive(Debug, Clone)]
pub struct Session {
    pub session_id: i64,
    pub user_name: String,
    pub space_name: Option<String>,
    pub graph_addr: Option<String>,
    pub timezone: Option<i32>,
}

#[derive(Debug, Clone)]
pub enum RoleType {
    GOD,
    ADMIN,
    DBA,
    USER,
    GUEST,
}

/// ClientSession saves those information, including who created it, executed queries,
/// space role, etc. One user corresponds to one ClientSession.
#[derive(Debug)]
pub struct ClientSession {
    session: Arc<Mutex<Session>>,
    space: Arc<Mutex<Option<SpaceInfo>>>,
    roles: Arc<Mutex<HashMap<i64, RoleType>>>,
    idle_start_time: Arc<Mutex<Instant>>,
    contexts: Arc<Mutex<HashMap<u32, String>>>, // Represents queries running in this session
}

impl ClientSession {
    pub fn new(session: Session) -> Arc<Self> {
        Arc::new(Self {
            session: Arc::new(Mutex::new(session)),
            space: Arc::new(Mutex::new(None)),
            roles: Arc::new(Mutex::new(HashMap::new())),
            idle_start_time: Arc::new(Mutex::new(Instant::now())),
            contexts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn id(&self) -> i64 {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .session_id
    }

    pub fn space(&self) -> Option<SpaceInfo> {
        self.space.lock().expect("Space lock was poisoned").clone()
    }

    pub fn set_space(&self, space: SpaceInfo) {
        *self.space.lock().expect("Space lock was poisoned") = Some(space);
    }

    pub fn space_name(&self) -> Option<String> {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .space_name
            .clone()
    }

    pub fn user(&self) -> String {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .user_name
            .clone()
    }

    pub fn roles(&self) -> HashMap<i64, RoleType> {
        self.roles.lock().expect("Roles lock was poisoned").clone()
    }

    pub fn role_with_space(&self, space: i64) -> Option<RoleType> {
        self.roles
            .lock()
            .expect("Roles lock was poisoned")
            .get(&space)
            .cloned()
    }

    pub fn is_god(&self) -> bool {
        self.roles
            .lock()
            .expect("Roles lock was poisoned")
            .values()
            .any(|role| matches!(role, RoleType::GOD))
    }

    pub fn set_role(&self, space: i64, role: RoleType) {
        self.roles
            .lock()
            .expect("Roles lock was poisoned")
            .insert(space, role);
    }

    pub fn idle_seconds(&self) -> u64 {
        self.idle_start_time
            .lock()
            .expect("Idle start time lock was poisoned")
            .elapsed()
            .as_secs()
    }

    pub fn charge(&self) {
        *self
            .idle_start_time
            .lock()
            .expect("Idle start time lock was poisoned") = Instant::now();
    }

    pub fn timezone(&self) -> Option<i32> {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .timezone
    }

    pub fn set_timezone(&self, timezone: i32) {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .timezone = Some(timezone);
    }

    pub fn graph_addr(&self) -> Option<String> {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .graph_addr
            .clone()
    }

    pub fn update_graph_addr(&self, host_addr: String) {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .graph_addr = Some(host_addr);
    }

    pub fn get_session(&self) -> Session {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .clone()
    }

    pub fn update_space_name(&self, space_name: String) {
        self.session
            .lock()
            .expect("Session lock was poisoned")
            .space_name = Some(space_name);
    }

    pub fn add_query(&self, ep_id: u32, query_context: String) {
        self.contexts
            .lock()
            .expect("Contexts lock was poisoned")
            .insert(ep_id, query_context);
    }

    pub fn delete_query(&self, ep_id: u32) {
        self.contexts
            .lock()
            .expect("Contexts lock was poisoned")
            .remove(&ep_id);
    }

    pub fn find_query(&self, ep_id: u32) -> bool {
        self.contexts
            .lock()
            .expect("Contexts lock was poisoned")
            .contains_key(&ep_id)
    }

    pub fn mark_query_killed(&self, ep_id: u32) {
        // In a real implementation, this would mark the query as killed in the context
        // For now, we'll just remove it
        self.contexts
            .lock()
            .expect("Contexts lock was poisoned")
            .remove(&ep_id);
    }

    pub fn mark_all_queries_killed(&self) {
        self.contexts
            .lock()
            .expect("Contexts lock was poisoned")
            .clear();
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
        assert!(!client_session.is_god());
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

        client_session.set_role(1, RoleType::ADMIN);
        assert!(matches!(
            client_session.role_with_space(1),
            Some(RoleType::ADMIN)
        ));
        assert!(client_session.role_with_space(2).is_none());

        // Test is_god function
        client_session.set_role(2, RoleType::GOD);
        assert!(client_session.is_god());
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

        // Charge the session (reset idle time)
        client_session.charge();
        assert!(client_session.idle_seconds() <= initial_idle); // Should be close to 0 after charge
    }
}
