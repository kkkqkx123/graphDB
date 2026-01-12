use crate::core::context::session::{SessionInfo, SessionStatus};
use crate::core::error::DBError;
use crate::core::Value;
use crate::utils::{safe_lock, safe_read, safe_write};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use uuid::Uuid;

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

/// A session containing variables and settings
#[derive(Debug, Clone)]
pub struct Session {
    pub session_info: SessionInfo,
    pub variables: Arc<RwLock<HashMap<String, Value>>>,
    pub settings: Arc<RwLock<HashMap<String, Value>>>,
}

impl Session {
    pub fn new(user_id: Option<String>, client_info: String, connection_info: String) -> Self {
        let session_info = SessionInfo::new(
            SessionId::new().to_string(),
            user_id.unwrap_or_else(|| "anonymous".to_string()),
            vec![], // 默认无角色
            "",     // 客户端IP
            0,      // 客户端端口
            client_info,
            connection_info,
        );

        Self {
            session_info,
            variables: Arc::new(RwLock::new(HashMap::new())),
            settings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if the session is still valid based on timeout
    pub fn is_valid(&self, timeout: Duration) -> bool {
        self.session_info.is_valid(timeout)
    }

    /// Update the last accessed time
    pub fn touch(&mut self) {
        self.session_info.touch();
    }

    /// Get a session variable
    pub fn get_variable(&self, key: &str) -> Result<Option<Value>, DBError> {
        let vars = safe_read(&self.variables)?;
        Ok(vars.get(key).cloned())
    }

    /// Set a session variable
    pub fn set_variable(&self, key: String, value: Value) -> Result<(), DBError> {
        let mut vars = safe_write(&self.variables)?;
        vars.insert(key, value);
        Ok(())
    }

    /// Remove a session variable
    pub fn remove_variable(&self, key: &str) -> Result<Option<Value>, DBError> {
        let mut vars = safe_write(&self.variables)?;
        Ok(vars.remove(key))
    }

    /// Get a session setting
    pub fn get_setting(&self, key: &str) -> Result<Option<Value>, DBError> {
        let settings = safe_read(&self.settings)?;
        Ok(settings.get(key).cloned())
    }

    /// Set a session setting
    pub fn set_setting(&self, key: String, value: Value) -> Result<(), DBError> {
        let mut settings = safe_write(&self.settings)?;
        settings.insert(key, value);
        Ok(())
    }

    /// Close the session
    pub fn close(&mut self) {
        self.session_info.status = SessionStatus::Closed;
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
    pub fn create_session(
        &self,
        user_id: Option<String>,
        client_info: String,
        connection_info: String,
    ) -> Result<SessionId, DBError> {
        let session = Session::new(user_id, client_info, connection_info);
        let session_id = SessionId(session.session_info.session_id.clone());

        let session = Arc::new(Mutex::new(session));
        {
            let mut sessions = safe_write(&self.sessions)?;
            sessions.insert(session_id.clone(), session);
        }

        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(
        &self,
        session_id: &SessionId,
    ) -> Result<Option<Arc<Mutex<Session>>>, DBError> {
        let sessions = safe_read(&self.sessions)?;
        Ok(sessions.get(session_id).cloned())
    }

    /// Check if a session exists and is valid
    pub fn is_valid_session(&self, session_id: &SessionId) -> Result<bool, DBError> {
        if let Some(session) = self.get_session(session_id)? {
            let session = safe_lock(&session)?;
            Ok(session.is_valid(self.default_session_timeout))
        } else {
            Ok(false)
        }
    }

    /// Update the last accessed time for a session
    pub fn touch_session(&self, session_id: &SessionId) -> Result<bool, DBError> {
        if let Some(session) = self.get_session(session_id)? {
            let mut session = safe_lock(&session)?;
            session.touch();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Remove an expired session
    pub fn remove_session(&self, session_id: &SessionId) -> Result<bool, DBError> {
        let mut sessions = safe_write(&self.sessions)?;
        Ok(sessions.remove(session_id).is_some())
    }

    /// Get session info by ID
    pub fn get_session_info(&self, session_id: &SessionId) -> Result<Option<SessionInfo>, DBError> {
        if let Some(session) = self.get_session(session_id)? {
            let session = safe_lock(&session)?;
            Ok(Some(session.session_info.clone()))
        } else {
            Ok(None)
        }
    }

    /// List all active sessions
    pub fn list_active_sessions(&self) -> Result<Vec<SessionInfo>, DBError> {
        let sessions = safe_read(&self.sessions)?;
        let mut active_sessions = Vec::new();

        for session in sessions.values() {
            if let Ok(session) = safe_lock(session) {
                if matches!(session.session_info.status, SessionStatus::Active)
                    && session.is_valid(self.default_session_timeout)
                {
                    active_sessions.push(session.session_info.clone());
                }
            }
        }

        Ok(active_sessions)
    }

    /// Clean up expired sessions
    pub fn cleanup_expired_sessions(&self) -> Result<(), DBError> {
        let mut sessions = safe_write(&self.sessions)?;
        sessions.retain(|_, session| {
            if let Ok(session) = safe_lock(session) {
                session.is_valid(self.default_session_timeout)
            } else {
                false // If lock is poisoned, remove the session
            }
        });
        Ok(())
    }

    /// Get the number of active sessions
    pub fn active_session_count(&self) -> Result<usize, DBError> {
        Ok(self.list_active_sessions()?.len())
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
        session
            .session_info
            .created_at
            .elapsed()
            .unwrap_or(Duration::from_secs(0))
    }

    /// Check if a session is about to expire
    pub fn is_about_to_expire(session: &Session, warning_threshold: Duration) -> bool {
        if let Ok(elapsed) = session.session_info.created_at.elapsed() {
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

        assert!(session.session_info.session_id.len() > 0);
        assert_eq!(session.session_info.username, "user123");

        // Check that session is initially active
        assert!(matches!(session.session_info.status, SessionStatus::Active));
    }

    #[test]
    fn test_session_variables() {
        let session = Session::new(None, "".to_string(), "".to_string());

        // Set a variable
        session
            .set_variable("test_key".to_string(), Value::Int(42))
            .expect("Failed to set session variable in test");

        // Get the variable
        let value = session
            .get_variable("test_key")
            .expect("Failed to get session variable in test");
        assert_eq!(value, Some(Value::Int(42)));

        // Remove the variable
        let removed_value = session
            .remove_variable("test_key")
            .expect("Failed to remove session variable in test");
        assert_eq!(removed_value, Some(Value::Int(42)));

        // Check that it's gone
        let value = session
            .get_variable("test_key")
            .expect("Failed to get session variable in test");
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let session_manager = SessionManager::new(Duration::from_secs(300)); // 5 minutes timeout

        // Create a session
        let session_id = session_manager
            .create_session(
                Some("user123".to_string()),
                "client_info".to_string(),
                "connection_info".to_string(),
            )
            .expect("Failed to create session in test");

        // Verify the session exists
        assert!(session_manager
            .is_valid_session(&session_id)
            .expect("Failed to check session validity in test"));

        // Get session info
        let info = session_manager
            .get_session_info(&session_id)
            .expect("Failed to get session info in test");
        assert!(info.is_some());
        assert_eq!(
            info.expect("Session info should exist").username,
            "user123".to_string()
        );

        // Touch the session to update last_accessed time
        assert!(session_manager
            .touch_session(&session_id)
            .expect("Failed to touch session in test"));

        // List active sessions
        let active_sessions = session_manager
            .list_active_sessions()
            .expect("Failed to list active sessions in test");
        assert_eq!(active_sessions.len(), 1);

        // Clean up
        session_manager
            .remove_session(&session_id)
            .expect("Failed to remove session in test");
        assert!(!session_manager
            .is_valid_session(&session_id)
            .expect("Failed to check session validity in test"));
    }

    #[test]
    fn test_session_timeout() {
        let mut session = Session::new(None, "".to_string(), "".to_string());

        // Initially, the session should be valid
        assert!(session.is_valid(Duration::from_secs(10)));

        // Modify the last_accessed time to be in the past
        session.session_info.last_accessed = SystemTime::now() - Duration::from_secs(15); // 15 seconds ago

        // Now it should be invalid with a 10-second timeout
        assert!(!session.is_valid(Duration::from_secs(10)));

        // But valid with a 20-second timeout
        assert!(session.is_valid(Duration::from_secs(20)));
    }
}
