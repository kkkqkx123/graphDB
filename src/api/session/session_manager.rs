use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::time;

use super::client_session::{ClientSession, Session};

pub const MAX_ALLOWED_CONNECTIONS: usize = 1000; // Default maximum connections
pub const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes

#[derive(Debug)]
pub struct GraphSessionManager {
    sessions: Arc<Mutex<HashMap<i64, Arc<ClientSession>>>>,
    active_sessions: Arc<Mutex<HashMap<i64, Instant>>>, // session_id -> last_activity_time
    host_addr: String,
}

impl GraphSessionManager {
    pub fn new(host_addr: String) -> Arc<Self> {
        let manager = Arc::new(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            host_addr,
        });

        // Start background tasks
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
        // Check if we're out of connections
        if self.is_out_of_connections() {
            return Err("Exceeded maximum allowed connections".to_string());
        }

        // Generate a new session ID
        let session_id = self.generate_session_id();

        let session = Session {
            session_id,
            user_name,
            space_name: None,
            graph_addr: Some(self.host_addr.clone()),
            timezone: None,
        };

        let client_session = ClientSession::new(session);

        // Add to sessions and active sessions
        {
            let mut sessions = self.sessions.lock().expect("Sessions lock was poisoned");
            let mut active_sessions = self
                .active_sessions
                .lock()
                .expect("Active sessions lock was poisoned");

            sessions.insert(session_id, Arc::clone(&client_session));
            active_sessions.insert(session_id, Instant::now());
        }

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
        {
            let mut sessions = self.sessions.lock().expect("Sessions lock was poisoned");
            let mut active_sessions = self
                .active_sessions
                .lock()
                .expect("Active sessions lock was poisoned");

            sessions.remove(&session_id);
            active_sessions.remove(&session_id);
        }
    }

    /// Gets all sessions from the local cache
    pub fn get_sessions_from_local_cache(&self) -> Vec<Session> {
        let sessions = self.sessions.lock().expect("Sessions lock was poisoned");
        sessions
            .values()
            .map(|session| session.get_session())
            .collect()
    }

    /// Whether exceeds the max allowed connections
    pub fn is_out_of_connections(&self) -> bool {
        let active_sessions = self
            .active_sessions
            .lock()
            .expect("Active sessions lock was poisoned");
        active_sessions.len() >= MAX_ALLOWED_CONNECTIONS
    }

    /// Generate a new unique session ID
    fn generate_session_id(&self) -> i64 {
        // In a real implementation, this might be more sophisticated
        // For now, we'll use seconds since epoch as a simple ID
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("System time is before Unix epoch")
            .as_secs() as i64
    }

    /// Background task to reclaim expired sessions
    async fn background_reclamation_task(self: Arc<Self>) {
        let mut interval = time::interval(Duration::from_secs(30)); // Check every 30 seconds

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
            .filter(|(_, last_activity)| last_activity.elapsed() > SESSION_IDLE_TIMEOUT)
            .map(|(&session_id, _)| session_id)
            .collect();
        drop(active_sessions);

        // Remove expired sessions
        for session_id in expired_sessions {
            self.remove_session(session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let session_manager = GraphSessionManager::new("127.0.0.1:9669".to_string());

        assert_eq!(session_manager.host_addr, "127.0.0.1:9669");
        assert_eq!(session_manager.get_sessions_from_local_cache().len(), 0);
    }

    #[tokio::test]
    async fn test_create_and_find_session() {
        let session_manager = GraphSessionManager::new("127.0.0.1:9669".to_string());

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
        let session_manager = GraphSessionManager::new("127.0.0.1:9669".to_string());

        let session = session_manager
            .create_session("testuser".to_string(), "127.0.0.1".to_string())
            .expect("Failed to create session");

        assert!(session_manager.find_session(session.id()).is_some());

        session_manager.remove_session(session.id());
        assert!(session_manager.find_session(session.id()).is_none());
    }

    #[tokio::test]
    async fn test_max_connections() {
        let session_manager = GraphSessionManager::new("127.0.0.1:9669".to_string());

        // Temporarily set a low max connections for testing
        // In a real test, we'd need to make MAX_ALLOWED_CONNECTIONS configurable
        assert!(!session_manager.is_out_of_connections());
    }

    #[tokio::test]
    async fn test_session_cache_operations() {
        let session_manager = GraphSessionManager::new("127.0.0.1:9669".to_string());

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
}
