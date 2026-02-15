use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub mod test_config;

/// 数据库配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 存储路径
    pub storage_path: String,
    /// 最大连接数
    pub max_connections: usize,
    /// 事务超时时间（秒）
    pub transaction_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
        }
    }
}

/// 日志配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogConfig {
    /// 日志级别
    pub level: String,
    /// 日志目录
    pub dir: String,
    /// 日志文件名
    pub file: String,
    /// 单个日志文件最大大小（字节）
    pub max_file_size: u64,
    /// 最大日志文件数量
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

/// 授权配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    /// 是否启用授权
    pub enable_authorize: bool,
    /// 登录失败次数限制（0表示不限制）
    pub failed_login_attempts: u32,
    /// 会话空闲超时时间（秒）
    pub session_idle_timeout_secs: u64,
    /// 是否强制修改默认密码（首次登录时）
    pub force_change_default_password: bool,
    /// 默认用户名
    pub default_username: String,
    /// 默认密码（仅在首次启动或单用户模式使用）
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

/// 初始化配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BootstrapConfig {
    /// 是否自动创建默认Space
    pub auto_create_default_space: bool,
    /// 默认Space名称
    pub default_space_name: String,
    /// 单用户模式（跳过认证，始终使用默认用户）
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

/// 优化器规则配置
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct OptimizerRulesConfig {
    /// 禁用的规则
    #[serde(default)]
    pub disabled_rules: Vec<String>,
    /// 启用的规则
    #[serde(default)]
    pub enabled_rules: Vec<String>,
}

/// 优化器配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OptimizerConfig {
    /// 最大迭代轮数
    pub max_iteration_rounds: usize,
    /// 最大探索轮数
    pub max_exploration_rounds: usize,
    /// 是否启用代价模型
    pub enable_cost_model: bool,
    /// 是否启用多计划
    pub enable_multi_plan: bool,
    /// 是否启用属性剪枝
    pub enable_property_pruning: bool,
    /// 是否启用自适应迭代
    pub enable_adaptive_iteration: bool,
    /// 稳定阈值
    pub stable_threshold: usize,
    /// 最小迭代轮数
    pub min_iteration_rounds: usize,
    /// 规则配置
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

/// 全局配置
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 日志配置
    pub log: LogConfig,
    /// 授权配置
    pub auth: AuthConfig,
    /// 初始化配置
    pub bootstrap: BootstrapConfig,
    /// 优化器配置
    pub optimizer: OptimizerConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            log: LogConfig::default(),
            auth: AuthConfig::default(),
            bootstrap: BootstrapConfig::default(),
            optimizer: OptimizerConfig::default(),
        }
    }
}

impl Config {
    /// 从文件加载配置
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.database.storage_path = Config::resolve_storage_path(&config.database.storage_path)?;
        Ok(config)
    }

    /// 保存配置到文件
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 解析存储路径（支持相对路径和 ~ 展开）
    fn resolve_storage_path(storage_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let path = PathBuf::from(storage_path);

        if path.is_absolute() {
            return Ok(storage_path.to_string());
        }

        if storage_path.starts_with('~') {
            if let Some(home_dir) = env::home_dir() {
                let relative_path = &storage_path[1..];
                let absolute_path = if relative_path.starts_with('/') || relative_path.starts_with('\\') {
                    home_dir.join(&relative_path[1..])
                } else {
                    home_dir.join(relative_path)
                };
                return Ok(absolute_path.to_string_lossy().into_owned());
            }
            return Err("无法获取用户主目录".into());
        }

        if let Ok(exe_path) = env::current_exe() {
            let exe_dir = exe_path
                .parent()
                .ok_or("无法获取可执行文件所在目录")?
                .to_path_buf();
            let absolute_path = exe_dir.join(&path);
            return Ok(absolute_path.to_string_lossy().into_owned());
        }

        Err("无法获取可执行文件路径".into())
    }

    /// 获取日志级别
    pub fn log_level(&self) -> &str {
        &self.log.level
    }

    /// 获取日志目录
    pub fn log_dir(&self) -> &str {
        &self.log.dir
    }

    /// 获取日志文件名
    pub fn log_file(&self) -> &str {
        &self.log.file
    }

    /// 获取主机地址
    pub fn host(&self) -> &str {
        &self.database.host
    }

    /// 获取端口
    pub fn port(&self) -> u16 {
        self.database.port
    }

    /// 获取存储路径
    pub fn storage_path(&self) -> &str {
        &self.database.storage_path
    }

    /// 获取最大连接数
    pub fn max_connections(&self) -> usize {
        self.database.max_connections
    }

    /// 获取事务超时时间
    pub fn transaction_timeout(&self) -> u64 {
        self.database.transaction_timeout
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
        assert_eq!(config.auth.enable_authorize, true);
        assert_eq!(config.bootstrap.auto_create_default_space, true);
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
transaction_timeout = 60

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

        let temp_dir = std::env::temp_dir();
        let temp_path = temp_dir.join("test_nested_config.toml");
        std::fs::write(&temp_path, config_content).expect("Failed to write config file");

        let config = Config::load(&temp_path).expect("Failed to load config");

        assert_eq!(config.database.host, "0.0.0.0");
        assert_eq!(config.database.port, 8080);
        assert_eq!(config.log.level, "debug");
        assert_eq!(config.auth.enable_authorize, false);
        assert_eq!(config.auth.default_username, "admin");
        assert_eq!(config.bootstrap.single_user_mode, true);
        assert_eq!(config.optimizer.max_iteration_rounds, 10);
        assert_eq!(config.optimizer.enable_cost_model, false);
        assert_eq!(config.optimizer.rules.disabled_rules.len(), 2);
        assert_eq!(config.optimizer.rules.enabled_rules.len(), 1);

        // 清理
        let _ = std::fs::remove_file(&temp_path);
    }
}
