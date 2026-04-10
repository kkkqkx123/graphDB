use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::search::config::FulltextConfig;
use vector_client::VectorClientConfig;

/// Database configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// Host address
    pub host: String,
    /// Port
    pub port: u16,
    /// Storage path
    pub storage_path: String,
    /// Maximum connections
    pub max_connections: usize,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),
            max_connections: 10,
        }
    }
}

/// Transaction configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionConfig {
    /// Default transaction timeout (seconds)
    pub default_timeout: u64,
    /// Maximum concurrent transactions
    pub max_concurrent_transactions: usize,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
            default_timeout: 30,
            max_concurrent_transactions: 1000,
        }
    }
}

/// Log configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    /// Log level
    pub level: String,
    /// Log directory
    pub dir: String,
    /// Log file name
    pub file: String,
    /// Maximum size of a single log file (bytes)
    pub max_file_size: u64,
    /// Maximum number of log files
    pub max_files: usize,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            dir: "logs".to_string(),
            file: "graphdb".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            max_files: 5,
        }
    }
}

/// Authorization configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    /// Whether to enable authorization
    pub enable_authorize: bool,
    /// Maximum failed login attempts (0 means unlimited)
    pub failed_login_attempts: u32,
    /// Session idle timeout (seconds)
    pub session_idle_timeout_secs: u64,
    /// Whether to force changing the default password (on first login)
    pub force_change_default_password: bool,
    /// Default username
    pub default_username: String,
    /// Default password (used only on first start or in single-user mode)
    pub default_password: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enable_authorize: true,
            failed_login_attempts: 5,
            session_idle_timeout_secs: 3600,
            force_change_default_password: true,
            default_username: "root".to_string(),
            default_password: "root".to_string(),
        }
    }
}

/// Bootstrap configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapConfig {
    /// Whether to automatically create the default Space
    pub auto_create_default_space: bool,
    /// Default Space name
    pub default_space_name: String,
    /// Single-user mode (skip authentication, always use the default user)
    pub single_user_mode: bool,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            auto_create_default_space: true,
            default_space_name: "default".to_string(),
            single_user_mode: false,
        }
    }
}

/// Optimizer rules configuration
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct OptimizerRulesConfig {
    /// Disabled rules
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    /// Enabled rules
    #[serde(default)]
    pub enabled_rules: Vec<String>,
}

/// Optimizer configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OptimizerConfig {
    /// Maximum iteration rounds
    pub max_iteration_rounds: usize,
    /// Maximum exploration rounds
    pub max_exploration_rounds: usize,
    /// Whether to enable cost model
    pub enable_cost_model: bool,
    /// Whether to enable multi-plan
    pub enable_multi_plan: bool,
    /// Whether to enable property pruning
    pub enable_property_pruning: bool,
    /// Whether to enable adaptive iteration
    pub enable_adaptive_iteration: bool,
    /// Stable threshold
    pub stable_threshold: usize,
    /// Minimum iteration rounds
    pub min_iteration_rounds: usize,
    /// Rules configuration
    #[serde(default)]
    pub rules: OptimizerRulesConfig,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            max_iteration_rounds: 5,
            max_exploration_rounds: 128,
            enable_cost_model: true,
            enable_multi_plan: true,
            enable_property_pruning: true,
            enable_adaptive_iteration: true,
            stable_threshold: 2,
            min_iteration_rounds: 1,
            rules: OptimizerRulesConfig::default(),
        }
    }
}

/// Monitoring configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MonitoringConfig {
    /// Whether to enable monitoring
    pub enabled: bool,
    /// Memory cache size (retains the most recent N queries)
    pub memory_cache_size: usize,
    /// Slow query threshold (milliseconds)
    pub slow_query_threshold_ms: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            memory_cache_size: 1000,
            slow_query_threshold_ms: 1000,
        }
    }
}

/// Global configuration
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    /// Database configuration
    pub database: DatabaseConfig,
    /// Transaction configuration
    #[serde(default)]
    pub transaction: TransactionConfig,
    /// Log configuration
    pub log: LogConfig,
    /// Authorization configuration
    pub auth: AuthConfig,
    /// Bootstrap configuration
    pub bootstrap: BootstrapConfig,
    /// Optimizer configuration
    pub optimizer: OptimizerConfig,
    /// Monitoring configuration
    #[serde(default)]
    pub monitoring: MonitoringConfig,
    /// Vector search configuration
    #[serde(default)]
    pub vector: VectorClientConfig,
    /// Fulltext search configuration
    #[serde(default)]
    pub fulltext: FulltextConfig,
}

impl Config {
    /// Load configuration from file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.database.storage_path = Config::resolve_storage_path(&config.database.storage_path)?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Resolve storage path (supports relative paths and ~ expansion)
    fn resolve_storage_path(storage_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = PathBuf::from(storage_path);

        if path.is_absolute() {
            return Ok(storage_path.to_string());
        }

        if let Some(relative_path) = storage_path.strip_prefix('~') {
            if let Some(home_dir) = env::home_dir() {
                let absolute_path =
                    if relative_path.starts_with('/') || relative_path.starts_with('\\') {
                        home_dir.join(&relative_path[1..])
                    } else {
                        home_dir.join(relative_path)
                    };
                return Ok(absolute_path.to_string_lossy().into_owned());
            }
            return Err("Failed to get user home directory".into());
        }

        if let Ok(exe_path) = env::current_exe() {
            let exe_dir = exe_path
                .parent()
                .ok_or("Failed to get executable directory")?
                .to_path_buf();
            let absolute_path = exe_dir.join(&path);
            return Ok(absolute_path.to_string_lossy().into_owned());
        }

        Err("Failed to get executable path".into())
    }

    /// Get log level
    pub fn log_level(&self) -> &str {
        &self.log.level
    }

    /// Get log directory
    pub fn log_dir(&self) -> &str {
        &self.log.dir
    }

    /// Get log file name
    pub fn log_file(&self) -> &str {
        &self.log.file
    }

    /// Get host address
    pub fn host(&self) -> &str {
        &self.database.host
    }

    /// Get port
    pub fn port(&self) -> u16 {
        self.database.port
    }

    /// Get storage path
    pub fn storage_path(&self) -> &str {
        &self.database.storage_path
    }

    /// Get maximum connections
    pub fn max_connections(&self) -> usize {
        self.database.max_connections
    }

    /// Get transaction timeout
    pub fn transaction_timeout(&self) -> u64 {
        self.transaction.default_timeout
    }

    /// Get maximum concurrent transactions
    pub fn max_concurrent_transactions(&self) -> usize {
        self.transaction.max_concurrent_transactions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.database.host, "127.0.0.1");
        assert_eq!(config.database.port, 9758);
        assert_eq!(config.log.level, "info");
        assert!(config.auth.enable_authorize);
        assert!(config.bootstrap.auto_create_default_space);
        assert_eq!(config.optimizer.max_iteration_rounds, 5);
    }

    #[test]
    fn test_config_load_save() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");

        let config = Config::default();
        let toml_content =
            toml::to_string_pretty(&config).expect("Failed to serialize config to TOML");
        temp_file
            .write_all(toml_content.as_bytes())
            .expect("Failed to write TOML content to temporary file");

        let loaded_config =
            Config::load(temp_file.path()).expect("Failed to load config from temporary file");
        assert_eq!(config.database.host, loaded_config.database.host);
        assert_eq!(config.database.port, loaded_config.database.port);
        assert_eq!(config.log.level, loaded_config.log.level);
    }

    #[test]
    fn test_nested_config_load() {
        let config_content = r#"
[database]
host = "0.0.0.0"
port = 8080
storage_path = "/tmp/graphdb"
max_connections = 100

[transaction]
default_timeout = 60
max_concurrent_transactions = 500

[log]
level = "debug"
dir = "/var/log/graphdb"
file = "graphdb"
max_file_size = 104857600
max_files = 10

[auth]
enable_authorize = false
failed_login_attempts = 3
session_idle_timeout_secs = 1800
force_change_default_password = false
default_username = "admin"
default_password = "admin123"

[bootstrap]
auto_create_default_space = false
default_space_name = "myspace"
single_user_mode = true

[optimizer]
max_iteration_rounds = 10
max_exploration_rounds = 256
enable_cost_model = false
enable_multi_plan = false
enable_property_pruning = false
enable_adaptive_iteration = false
stable_threshold = 5
min_iteration_rounds = 2

[optimizer.rules]
disabled_rules = ["FilterPushDownRule", "PredicatePushDownRule"]
enabled_rules = ["RemoveUselessNodeRule"]
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temporary file");
        temp_file
            .write_all(config_content.as_bytes())
            .expect("Failed to write config file");

        let config = Config::load(temp_file.path()).expect("Failed to load config");

        assert_eq!(config.database.host, "0.0.0.0");
        assert_eq!(config.database.port, 8080);
        assert_eq!(config.transaction.default_timeout, 60);
        assert_eq!(config.transaction.max_concurrent_transactions, 500);
        assert_eq!(config.log.level, "debug");
        assert!(!config.auth.enable_authorize);
        assert_eq!(config.auth.default_username, "admin");
        assert!(config.bootstrap.single_user_mode);
        assert_eq!(config.optimizer.max_iteration_rounds, 10);
        assert!(!config.optimizer.enable_cost_model);
        assert_eq!(config.optimizer.rules.disabled_rules.len(), 2);
        assert_eq!(config.optimizer.rules.enabled_rules.len(), 1);
    }
}
