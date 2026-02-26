use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::config::AuthConfig;
use crate::core::error::AuthResult;

/// 认证 trait
pub trait Authenticator: Send + Sync {
    fn authenticate(&self, username: &str, password: &str) -> AuthResult<()>;
}

/// 登录失败记录
#[derive(Debug, Clone)]
struct LoginAttempt {
    /// 剩余尝试次数
    remaining_attempts: u32,
}

/// 用户验证回调函数类型
pub type UserVerifier = Arc<dyn Fn(&str, &str) -> AuthResult<bool> + Send + Sync>;

/// 密码认证器 - 支持登录失败限制和账户锁定
pub struct PasswordAuthenticator {
    /// 用户验证回调
    user_verifier: UserVerifier,
    config: AuthConfig,
    /// 用户登录尝试记录
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
}

impl PasswordAuthenticator {
    pub fn new<F>(user_verifier: F, config: AuthConfig) -> Self 
    where
        F: Fn(&str, &str) -> AuthResult<bool> + Send + Sync + 'static,
    {
        Self {
            user_verifier: Arc::new(user_verifier),
            config,
            login_attempts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 创建默认的密码认证器（支持配置的用户名和密码）
    pub fn new_default(config: AuthConfig) -> Self {
        let default_username = config.default_username.clone();
        let default_password = config.default_password.clone();
        
        Self::new(
            move |username: &str, password: &str| {
                // 使用配置的默认用户名和密码
                if username == default_username && password == default_password {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            config,
        )
    }

    /// 记录登录失败
    fn record_failed_attempt(&self, username: &str) {
        // 如果未启用登录失败限制，直接返回
        if self.config.failed_login_attempts == 0 {
            return;
        }

        let mut attempts = self.login_attempts.write();
        
        let attempt = attempts.entry(username.to_string()).or_insert(LoginAttempt {
            remaining_attempts: self.config.failed_login_attempts,
        });

        // 减少剩余尝试次数
        if attempt.remaining_attempts > 0 {
            attempt.remaining_attempts -= 1;
        }
    }

    /// 重置登录尝试记录（登录成功时调用）
    fn reset_attempts(&self, username: &str) {
        let mut attempts = self.login_attempts.write();
        attempts.remove(username);
    }

    /// 验证用户密码
    fn verify_password(&self, username: &str, password: &str) -> AuthResult<bool> {
        (self.user_verifier)(username, password)
    }
}

impl Authenticator for PasswordAuthenticator {
    fn authenticate(&self, username: &str, password: &str) -> AuthResult<()> {
        use crate::core::error::AuthError;

        // 检查是否启用授权
        if !self.config.enable_authorize {
            return Ok(());
        }

        if username.is_empty() || password.is_empty() {
            return Err(AuthError::EmptyCredentials);
        }

        // 验证密码
        match self.verify_password(username, password) {
            Ok(true) => {
                // 登录成功，重置尝试记录
                self.reset_attempts(username);
                Ok(())
            }
            Ok(false) => {
                // 登录失败，记录尝试
                self.record_failed_attempt(username);
                
                let attempts = self.login_attempts.read();
                if let Some(attempt) = attempts.get(username) {
                    if attempt.remaining_attempts > 0 {
                        return Err(AuthError::InvalidCredentials);
                    } else {
                        return Err(AuthError::MaxAttemptsExceeded);
                    }
                }
                
                Err(AuthError::InvalidCredentials)
            }
            Err(e) => Err(e),
        }
    }
}

/// 认证器工厂
pub struct AuthenticatorFactory;

impl AuthenticatorFactory {
    /// 创建密码认证器
    pub fn create<F>(
        config: &AuthConfig,
        user_verifier: F,
    ) -> PasswordAuthenticator
    where
        F: Fn(&str, &str) -> AuthResult<bool> + Send + Sync + 'static,
    {
        PasswordAuthenticator::new(user_verifier, config.clone())
    }

    /// 创建默认的密码认证器
    pub fn create_default(config: &AuthConfig) -> PasswordAuthenticator {
        PasswordAuthenticator::new_default(config.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> AuthConfig {
        AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 3,
            session_idle_timeout_secs: 3600,
            default_username: "test".to_string(),
            default_password: "test123".to_string(),
            force_change_default_password: true,
        }
    }

    #[test]
    fn test_password_authenticator_success() {
        let config = create_test_config();
        let auth = PasswordAuthenticator::new(
            |_username: &str, _password: &str| Ok(true),
            config,
        );

        assert!(auth.authenticate("user", "pass").is_ok());
    }

    #[test]
    fn test_password_authenticator_failure() {
        let config = create_test_config();
        let auth = PasswordAuthenticator::new(
            |_username: &str, _password: &str| Ok(false),
            config,
        );

        assert!(auth.authenticate("user", "wrong_pass").is_err());
    }

    #[test]
    fn test_password_authenticator_default() {
        let config = AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 0, // 禁用登录限制
            session_idle_timeout_secs: 3600,
            default_username: "admin".to_string(),
            default_password: "admin123".to_string(),
            force_change_default_password: false,
        };

        let auth = PasswordAuthenticator::new_default(config);

        // 使用正确的默认凭据
        assert!(auth.authenticate("admin", "admin123").is_ok());
        // 使用错误的凭据
        assert!(auth.authenticate("admin", "wrong").is_err());
        assert!(auth.authenticate("user", "admin123").is_err());
    }

    #[test]
    fn test_login_attempt_limit() {
        let config = AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 2, // 最多2次失败
            session_idle_timeout_secs: 3600,
            default_username: "test".to_string(),
            default_password: "test123".to_string(),
            force_change_default_password: false,
        };

        let auth = PasswordAuthenticator::new(
            |_username: &str, _password: &str| Ok(false),
            config,
        );

        // 第一次失败 - 还剩1次
        let result1 = auth.authenticate("user", "wrong");
        assert!(result1.is_err());
        assert!(result1.unwrap_err().to_string().contains("还剩 1 次"));

        // 第二次失败 - 达到最大尝试次数
        let result2 = auth.authenticate("user", "wrong");
        assert!(result2.is_err());
        assert!(result2.unwrap_err().to_string().contains("最大尝试次数"));

        // 第三次失败 - 仍然显示达到最大尝试次数
        let result3 = auth.authenticate("user", "wrong");
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("最大尝试次数"));
    }

    #[test]
    fn test_successful_login_resets_attempts() {
        // 使用Arc<AtomicBool>来在闭包中共享可变状态
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let config = AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 2,
            session_idle_timeout_secs: 3600,
            default_username: "test".to_string(),
            default_password: "test123".to_string(),
            force_change_default_password: false,
        };

        let success = Arc::new(AtomicBool::new(false));
        let success_clone = success.clone();
        
        let auth = PasswordAuthenticator::new(
            move |_username: &str, _password: &str| {
                if success_clone.load(Ordering::SeqCst) {
                    Ok(true)
                } else {
                    Ok(false)
                }
            },
            config,
        );

        // 第一次失败
        assert!(auth.authenticate("user", "wrong").is_err());

        // 成功登录应该重置失败计数
        success.store(true, Ordering::SeqCst);
        assert!(auth.authenticate("user", "correct").is_ok());

        // 再次失败，应该重新计数
        success.store(false, Ordering::SeqCst);
        assert!(auth.authenticate("user", "wrong").is_err());
        // 还有一次机会
        assert!(auth.authenticate("user", "wrong").is_err());
    }

    #[test]
    fn test_authenticator_factory() {
        let config = AuthConfig {
            enable_authorize: true,
            failed_login_attempts: 0,
            session_idle_timeout_secs: 3600,
            default_username: "test".to_string(),
            default_password: "test123".to_string(),
            force_change_default_password: false,
        };

        let _auth = AuthenticatorFactory::create(
            &config,
            |_username: &str, _password: &str| Ok(true),
        );
        // 验证创建成功（不再返回Result，直接创建成功）

        let _auth_default = AuthenticatorFactory::create_default(&config);
        // 验证创建成功
    }
}
