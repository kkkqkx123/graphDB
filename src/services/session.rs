use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, Duration, UNIX_EPOCH};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::core::Value;

/// Unique identifier for a session
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        SessionId(Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Idle,
    Expired,
    Closed,
}

/// Information about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: SessionId,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub status: SessionStatus,
    pub user_id: Option<String>,
    pub client_info: String,
    pub connection_info: String,
}

/// A session containing variables and settings
#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub created_at: SystemTime,
    pub last_accessed: SystemTime,
    pub status: SessionStatus,
    pub user_id: Option<String>,
    pub variables: Arc<RwLock<HashMap<String, Value>>>,
    pub settings: Arc<RwLock<HashMap<String, Value>>>,
    pub client_info: String,
    pub connection_info: String,
}

impl Session {
    pub fn new(user_id: Option<String>, client_info: String, connection_info: String) -> Self {
        Self {
            id: SessionId::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            status: SessionStatus::Active,
            user_id,
            variables: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(HashMap::new())),
            client_info,
            connection_info,
        }
    }

    /// Check if the session is still valid based on timeout
    pub fn is_valid(&self, timeout: Duration) -> bool {
        if let Ok(elapsed) = self.last_accessed.elapsed() {
            elapsed < timeout && matches!(self.status, SessionStatus::Active | SessionStatus::Idle)
        } else {
            false
        }
    }

    /// Update the last accessed time
    pub fn touch(&mut self) {
        self.last_accessed = SystemTime::now();
        if matches!(self.status, SessionStatus::Expired | SessionStatus::Closed) {
            self.status = SessionStatus::Active;
        }
    }

    /// Get a session variable
    pub fn get_variable(&self, key: &str) -> Option<Value> {
        let vars = self.variables.read().unwrap();
        vars.get(key).cloned()
    }

    /// Set a session variable
    pub fn set_variable(&self, key: String, value: Value) {
        let mut vars = self.variables.write().unwrap();
        vars.insert(key, value);
    }

    /// Remove a session variable
    pub fn remove_variable(&self, key: &str) -> Option<Value> {
        let mut vars = self.variables.write().unwrap();
        vars.remove(key)
    }

    /// Get a session setting
    pub fn get_setting(&self, key: &str) -> Option<Value> {
        let settings = self.settings.read().unwrap();
        settings.get(key).cloned()
    }

    /// Set a session setting
    pub fn set_setting(&self, key: String, value: Value) {
        let mut settings = self.settings.write().unwrap();
        settings.insert(key, value);
    }

    /// Get session info
    pub fn info(&self) -> SessionInfo {
        SessionInfo {
            id: self.id.clone(),
            created_at: self.created_at,
            last_accessed: self.last_accessed,
            status: self.status.clone(),
            user_id: self.user_id.clone(),
            client_info: self.client_info.clone(),
            connection_info: self.connection_info.clone(),
        }
    }

    /// Close the session
    pub fn close(&mut self) {
        self.status = SessionStatus::Closed;
    }
}

/// Session manager to handle multiple sessions
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, Arc<Mutex<Session>>>>>,
    default_session_timeout: Duration,
}

impl SessionManager {
    pub fn new(default_session_timeout: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_session_timeout,
        }
    }

    /// Create a new session
    pub fn create_session(&self, user_id: Option<String>, client_info: String, connection_info: String) -> SessionId {
        let mut session = Session::new(user_id, client_info, connection_info);
        let session_id = session.id.clone();
        
        let session = Arc::new(Mutex::new(session));
        {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session_id.clone(), session);
        }
        
        session_id
    }

    /// Get a session by ID
    pub fn get_session(&self, session_id: &SessionId) -> Option<Arc<Mutex<Session>>> {
        let sessions = self.sessions.read().unwrap();
        sessions.get(session_id).cloned()
    }

    /// Check if a session exists and is valid
    pub fn is_valid_session(&self, session_id: &SessionId) -> bool {
        if let Some(session) = self.get_session(session_id) {
            let session = session.lock().unwrap();
            session.is_valid(self.default_session_timeout)
        } else {
            false
        }
    }

    /// Update the last accessed time for a session
    pub fn touch_session(&self, session_id: &SessionId) -> bool {
        if let Some(session) = self.get_session(session_id) {
            let mut session = session.lock().unwrap();
            session.touch();
            true
        } else {
            false
        }
    }

    /// Remove an expired session
    pub fn remove_session(&self, session_id: &SessionId) -> bool {
        let mut sessions = self.sessions.write().unwrap();
        sessions.remove(session_id).is_some()
    }

    /// Get session info by ID
    pub fn get_session_info(&self, session_id: &SessionId) -> Option<SessionInfo> {
        if let Some(session) = self.get_session(session_id) {
            let session = session.lock().unwrap();
            Some(session.info())
        } else {
            None
        }
    }

    /// List all active sessions
    pub fn list_active_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().unwrap();
        sessions
            .values()
            .filter_map(|session| {
                let session = session.lock().unwrap();
                if matches!(session.status, SessionStatus::Active) && 
                   session.is_valid(self.default_session_timeout) {
                    Some(session.info())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| {
            let session = session.lock().unwrap();
            session.is_valid(self.default_session_timeout)
        });
    }

    /// Get the number of active sessions
    pub fn active_session_count(&self) -> usize {
        self.list_active_sessions().len()
    }
}

/// Session configuration
#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub default_timeout: Duration,
    pub max_sessions: usize,
    pub enable_encryption: bool,
    pub enable_compression: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30 * 60), // 30 minutes
            max_sessions: 1000,
            enable_encryption: false,
            enable_compression: false,
        }
    }
}

/// Session utilities
pub mod session_utils {
    use super::*;

    /// Create a session ID from a string (for testing purposes)
    pub fn create_session_id_from_str(id: &str) -> SessionId {
        SessionId(id.to_string())
    }

    /// Check if a session ID is valid (well-formed)
    pub fn is_valid_session_id(session_id: &SessionId) -> bool {
        !session_id.as_str().is_empty()
    }

    /// Get the age of a session
    pub fn session_age(session: &Session) -> Duration {
        session.created_at.elapsed().unwrap_or(Duration::from_secs(0))
    }

    /// Check if a session is about to expire
    pub fn is_about_to_expire(session: &Session, warning_threshold: Duration) -> bool {
        if let Ok(elapsed) = session.created_at.elapsed() {
            let remaining = self::default_session_timeout().saturating_sub(elapsed);
            remaining < warning_threshold
        } else {
            true // If there's an error, treat as about to expire
        }
    }

    /// Get the default session timeout
    fn default_session_timeout() -> Duration {
        Duration::from_secs(30 * 60) // 30 minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_id() {
        let session_id = SessionId::new();
        assert!(!session_id.as_str().is_empty());
        
        let session_id_str = session_id.to_string();
        assert_eq!(session_id.as_str(), session_id_str);
    }

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            Some("user123".to_string()),
            "client_info".to_string(),
            "connection_info".to_string(),
        );
        
        assert!(session.id.as_str().len() > 0);
        assert_eq!(session.user_id, Some("user123".to_string()));
        
        // Check that session is initially active
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[test]
    fn test_session_variables() {
        let session = Session::new(None, "".to_string(), "".to_string());
        
        // Set a variable
        session.set_variable("test_key".to_string(), Value::Int(42));
        
        // Get the variable
        let value = session.get_variable("test_key");
        assert_eq!(value, Some(Value::Int(42)));
        
        // Remove the variable
        let removed_value = session.remove_variable("test_key");
        assert_eq!(removed_value, Some(Value::Int(42)));
        
        // Check that it's gone
        let value = session.get_variable("test_key");
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let session_manager = SessionManager::new(Duration::from_secs(300)); // 5 minutes timeout
        
        // Create a session
        let session_id = session_manager.create_session(
            Some("user123".to_string()),
            "client_info".to_string(),
            "connection_info".to_string(),
        );
        
        // Verify the session exists
        assert!(session_manager.is_valid_session(&session_id));
        
        // Get session info
        let info = session_manager.get_session_info(&session_id);
        assert!(info.is_some());
        assert_eq!(info.as_ref().unwrap().user_id, Some("user123".to_string()));
        
        // Touch the session to update last_accessed time
        assert!(session_manager.touch_session(&session_id));
        
        // List active sessions
        let active_sessions = session_manager.list_active_sessions();
        assert_eq!(active_sessions.len(), 1);
        
        // Clean up
        session_manager.remove_session(&session_id);
        assert!(!session_manager.is_valid_session(&session_id));
    }

    #[test]
    fn test_session_timeout() {
        let mut session = Session::new(None, "".to_string(), "".to_string());
        
        // Initially, the session should be valid
        assert!(session.is_valid(Duration::from_secs(10)));
        
        // Modify the last_accessed time to be in the past
        session.last_accessed = SystemTime::now() - Duration::from_secs(15); // 15 seconds ago
        
        // Now it should be invalid with a 10-second timeout
        assert!(!session.is_valid(Duration::from_secs(10)));
        
        // But valid with a 20-second timeout
        assert!(session.is_valid(Duration::from_secs(20)));
    }
}