use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub trait Authenticator: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> Result<()>;
}

pub struct PasswordAuthenticator {
    users: Arc<RwLock<HashMap<String, String>>>,
}

impl PasswordAuthenticator {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        users.insert("root".to_string(), "root".to_string());
        users.insert("nebula".to_string(), "nebula".to_string());

        Self {
            users: Arc::new(RwLock::new(users)),
        }
    }

    pub fn add_user(&self, username: String, password: String) -> Result<()> {
        let mut users = self.users.write().map_err(|e| anyhow!("获取写锁失败: {}", e))?;
        users.insert(username, password);
        Ok(())
    }

    pub fn remove_user(&self, username: &str) -> Result<()> {
        let mut users = self.users.write().map_err(|e| anyhow!("获取写锁失败: {}", e))?;
        users.remove(username);
        Ok(())
    }

    pub fn verify_password(&self, username: &str, password: &str) -> bool {
        let users = self.users.read().expect("获取读锁失败");
        users.get(username).map_or(false, |stored| stored == password)
    }
}

impl Authenticator for PasswordAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> Result<()> {
        if username.is_empty() || password.is_empty() {
            return Err(anyhow!("用户名或密码不能为空"));
        }

        if self.verify_password(username, password) {
            Ok(())
        } else {
            Err(anyhow!("用户名或密码错误"))
        }
    }
}

impl Default for PasswordAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_authenticator_creation() {
        let auth = PasswordAuthenticator::new();
        assert!(auth.verify_password("root", "root"));
        assert!(auth.verify_password("nebula", "nebula"));
    }

    #[test]
    fn test_authenticate_success() {
        let auth = PasswordAuthenticator::new();
        let result = auth.authenticate("root", "root");
        assert!(result.is_ok());
    }

    #[test]
    fn test_authenticate_failure() {
        let auth = PasswordAuthenticator::new();
        let result = auth.authenticate("root", "wrong_password");
        assert!(result.is_err());
    }

    #[test]
    fn test_authenticate_empty_credentials() {
        let auth = PasswordAuthenticator::new();
        let result = auth.authenticate("", "password");
        assert!(result.is_err());

        let result = auth.authenticate("root", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_add_user() {
        let auth = PasswordAuthenticator::new();
        let result = auth.add_user("testuser".to_string(), "testpass".to_string());
        assert!(result.is_ok());
        assert!(auth.verify_password("testuser", "testpass"));
    }

    #[test]
    fn test_remove_user() {
        let auth = PasswordAuthenticator::new();
        auth.add_user("testuser".to_string(), "testpass".to_string()).unwrap();
        assert!(auth.verify_password("testuser", "testpass"));

        let result = auth.remove_user("testuser");
        assert!(result.is_ok());
        assert!(!auth.verify_password("testuser", "testpass"));
    }
}
