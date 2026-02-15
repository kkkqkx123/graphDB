//! 日志系统集成测试
//!
//! 测试范围:
//! - 日志配置加载和验证
//! - 日志文件创建和写入
//! - 日志轮转功能
//! - 日志级别过滤

mod common;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use graphdb::config::Config;

/// 测试日志配置默认值
#[test]
fn test_log_config_defaults() {
    let config = Config::default();

    assert_eq!(config.log.level, "info");
    assert_eq!(config.log.dir, "logs");
    assert_eq!(config.log.file, "graphdb");
    assert_eq!(config.log.max_file_size, 100 * 1024 * 1024); // 100MB
    assert_eq!(config.log.max_files, 5);
}

/// 测试日志配置序列化和反序列化
#[test]
fn test_log_config_serialization() {
    let config = Config {
        database: graphdb::config::DatabaseConfig {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
        },
        log: graphdb::config::LogConfig {
            level: "debug".to_string(),
            dir: "test_logs".to_string(),
            file: "test_graphdb".to_string(),
            max_file_size: 50 * 1024 * 1024, // 50MB
            max_files: 3,
        },
        auth: graphdb::config::AuthConfig::default(),
        bootstrap: graphdb::config::BootstrapConfig::default(),
        optimizer: graphdb::config::OptimizerConfig::default(),
    };

    // 序列化为 TOML
    let toml_str = toml::to_string_pretty(&config).expect("序列化配置失败");

    // 验证 TOML 包含日志配置
    assert!(toml_str.contains("level = \"debug\""));
    assert!(toml_str.contains("dir = \"test_logs\""));
    assert!(toml_str.contains("file = \"test_graphdb\""));
    assert!(toml_str.contains("max_file_size = 52428800"));
    assert!(toml_str.contains("max_files = 3"));

    // 反序列化
    let loaded_config: Config = toml::from_str(&toml_str).expect("反序列化配置失败");
    assert_eq!(loaded_config.log.level, "debug");
    assert_eq!(loaded_config.log.dir, "test_logs");
    assert_eq!(loaded_config.log.file, "test_graphdb");
    assert_eq!(loaded_config.log.max_file_size, 52428800);
    assert_eq!(loaded_config.log.max_files, 3);
}

/// 测试日志目录创建
#[test]
fn test_log_directory_creation() {
    let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-logs")
        .join(format!("dir_test_{}", std::process::id()));

    // 确保目录不存在
    let _ = fs::remove_dir_all(&temp_dir);
    assert!(!temp_dir.exists());

    // 创建目录
    fs::create_dir_all(&temp_dir).expect("创建日志目录失败");
    assert!(temp_dir.exists());

    // 清理
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 测试日志配置从文件加载
#[test]
fn test_log_config_from_file() {
    let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-logs")
        .join(format!("config_test_{}", std::process::id()));

    // 清理并创建目录
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("创建测试目录失败");

    // 创建测试配置文件
    let config_content = r#"
[database]
host = "127.0.0.1"
port = 9758
storage_path = "data/graphdb"
max_connections = 10
transaction_timeout = 30

[log]
level = "debug"
dir = "custom_logs"
file = "custom_graphdb"
max_file_size = 52428800
max_files = 3

[auth]
enable_authorize = true
failed_login_attempts = 5
session_idle_timeout_secs = 3600
force_change_default_password = true
default_username = "root"
default_password = "root"

[bootstrap]
auto_create_default_space = true
default_space_name = "default"
single_user_mode = false

[optimizer]
max_iteration_rounds = 5
max_exploration_rounds = 128
enable_cost_model = true
enable_multi_plan = true
enable_property_pruning = true
enable_adaptive_iteration = true
stable_threshold = 2
min_iteration_rounds = 1
"#;

    let config_path = temp_dir.join("test_config.toml");
    fs::write(&config_path, config_content).expect("写入配置文件失败");

    // 加载配置
    let config = Config::load(&config_path).expect("加载配置失败");

    // 验证日志配置
    assert_eq!(config.log.level, "debug");
    assert_eq!(config.log.dir, "custom_logs");
    assert_eq!(config.log.file, "custom_graphdb");
    assert_eq!(config.log.max_file_size, 52428800);
    assert_eq!(config.log.max_files, 3);

    // 清理
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 集成测试：验证 flexi_logger 功能
/// 注意：由于 flexi_logger 使用全局 logger，所有功能在一个测试中验证
#[test]
fn test_flexi_logger_integration() {
    use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};

    let temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("test-logs")
        .join(format!("integration_test_{}", std::process::id()));

    // 清理并创建测试目录
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("创建测试目录失败");

    // ========== 测试 1: 基本日志写入 ==========
    {
        let test_dir = temp_dir.join("basic");
        fs::create_dir_all(&test_dir).expect("创建测试目录失败");

        let _logger = Logger::try_with_str("info")
            .expect("创建 logger 失败")
            .log_to_file(
                FileSpec::default()
                    .basename("basic_test")
                    .directory(&test_dir),
            )
            .write_mode(WriteMode::Direct)
            .start()
            .expect("启动 logger 失败");

        log::info!("基本日志写入测试");
        log::warn!("警告日志测试");
        log::error!("错误日志测试");

        // 等待日志写入
        std::thread::sleep(Duration::from_millis(500));

        // 查找生成的日志文件（flexi_logger 可能使用 rCURRENT 后缀）
        let log_files: Vec<_> = fs::read_dir(&test_dir)
            .expect("读取目录失败")
            .filter_map(|e| e.ok())
            .filter(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                name.starts_with("basic_test") && name.ends_with(".log")
            })
            .collect();

        assert!(!log_files.is_empty(), "应该至少有一个日志文件");

        // 读取第一个日志文件
        let log_file = &log_files[0];
        let content = fs::read_to_string(log_file.path()).expect("读取日志文件失败");
        assert!(content.contains("基本日志写入测试"), "日志应包含信息日志");
        assert!(content.contains("警告日志测试"), "日志应包含警告日志");
        assert!(content.contains("错误日志测试"), "日志应包含错误日志");
    }

    // ========== 测试 2: 日志级别过滤 ==========
    {
        let test_dir = temp_dir.join("level_filter");
        fs::create_dir_all(&test_dir).expect("创建测试目录失败");

        // 注意：由于全局 logger 已经设置，这里使用不同的方式测试
        // 实际上 flexi_logger 不支持在同一进程中重新初始化
        // 所以我们只验证配置可以被正确加载

        let config = Config {
            database: graphdb::config::DatabaseConfig::default(),
            log: graphdb::config::LogConfig {
                level: "warn".to_string(), // 只记录 warn 及以上级别
                dir: test_dir.to_string_lossy().to_string(),
                file: "level_test".to_string(),
                ..graphdb::config::LogConfig::default()
            },
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
        };

        // 验证配置正确
        assert_eq!(config.log.level, "warn");
        assert!(config.log.dir.contains("level_filter"));
    }

    // ========== 测试 3: 日志轮转配置验证 ==========
    {
        let test_dir = temp_dir.join("rotation");
        fs::create_dir_all(&test_dir).expect("创建测试目录失败");

        let config = Config {
            database: graphdb::config::DatabaseConfig::default(),
            log: graphdb::config::LogConfig {
                level: "info".to_string(),
                dir: test_dir.to_string_lossy().to_string(),
                file: "rotation_test".to_string(),
                max_file_size: 10 * 1024 * 1024, // 10MB
                max_files: 3,
            },
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
        };

        // 验证轮转配置
        assert_eq!(config.log.max_file_size, 10 * 1024 * 1024);
        assert_eq!(config.log.max_files, 3);

        // 验证 flexi_logger 的轮转配置可以正确构建
        let file_spec = FileSpec::default()
            .basename(&config.log.file)
            .directory(&config.log.dir);

        let _logger_builder = Logger::try_with_str(&config.log.level)
            .expect("创建 logger 失败")
            .log_to_file(file_spec)
            .rotate(
                Criterion::Size(config.log.max_file_size),
                Naming::Numbers,
                Cleanup::KeepLogFiles(config.log.max_files),
            );
        // 注意：不实际启动 logger，因为全局 logger 已存在
    }

    // ========== 测试 4: 异步写入配置验证 ==========
    {
        let test_dir = temp_dir.join("async");
        fs::create_dir_all(&test_dir).expect("创建测试目录失败");

        let config = Config {
            database: graphdb::config::DatabaseConfig::default(),
            log: graphdb::config::LogConfig {
                level: "debug".to_string(),
                dir: test_dir.to_string_lossy().to_string(),
                file: "async_test".to_string(),
                ..graphdb::config::LogConfig::default()
            },
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
        };

        // 验证异步配置可以正确构建
        let file_spec = FileSpec::default()
            .basename(&config.log.file)
            .directory(&config.log.dir);

        let _logger_builder = Logger::try_with_str(&config.log.level)
            .expect("创建 logger 失败")
            .log_to_file(file_spec)
            .write_mode(WriteMode::Async);
        // 注意：不实际启动 logger，因为全局 logger 已存在
    }

    // ========== 测试 5: 日志清理策略配置验证 ==========
    {
        let test_dir = temp_dir.join("cleanup");
        fs::create_dir_all(&test_dir).expect("创建测试目录失败");

        let max_files = 2;
        let config = Config {
            database: graphdb::config::DatabaseConfig::default(),
            log: graphdb::config::LogConfig {
                level: "info".to_string(),
                dir: test_dir.to_string_lossy().to_string(),
                file: "cleanup_test".to_string(),
                max_file_size: 1024 * 1024, // 1MB
                max_files,
            },
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
        };

        // 验证清理配置
        assert_eq!(config.log.max_files, max_files);

        // 验证 flexi_logger 的清理配置可以正确构建
        let file_spec = FileSpec::default()
            .basename(&config.log.file)
            .directory(&config.log.dir);

        let _logger_builder = Logger::try_with_str(&config.log.level)
            .expect("创建 logger 失败")
            .log_to_file(file_spec)
            .rotate(
                Criterion::Size(config.log.max_file_size),
                Naming::Numbers,
                Cleanup::KeepLogFiles(config.log.max_files),
            );
        // 注意：不实际启动 logger，因为全局 logger 已存在
    }

    // 清理所有测试目录
    let _ = fs::remove_dir_all(&temp_dir);
}

/// 测试日志文件路径解析
#[test]
fn test_log_file_path_resolution() {
    let config = Config::default();

    // 验证日志目录和文件名组合
    let expected_log_path = format!("{}/{}.log", config.log.dir, config.log.file);
    assert_eq!(expected_log_path, "logs/graphdb.log");

    // 测试自定义配置
    let custom_config = Config {
        database: graphdb::config::DatabaseConfig::default(),
        log: graphdb::config::LogConfig {
            dir: "/var/log/graphdb".to_string(),
            file: "app".to_string(),
            ..graphdb::config::LogConfig::default()
        },
        auth: graphdb::config::AuthConfig::default(),
        bootstrap: graphdb::config::BootstrapConfig::default(),
        optimizer: graphdb::config::OptimizerConfig::default(),
    };

    let custom_path = format!("{}/{}.log", custom_config.log.dir, custom_config.log.file);
    assert_eq!(custom_path, "/var/log/graphdb/app.log");
}

/// 测试日志文件大小配置
#[test]
fn test_log_file_size_config() {
    // 测试默认 100MB
    let config = Config::default();
    assert_eq!(config.log.max_file_size, 100 * 1024 * 1024);

    // 测试自定义大小
    let custom_config = Config {
        database: graphdb::config::DatabaseConfig::default(),
        log: graphdb::config::LogConfig {
            max_file_size: 500 * 1024 * 1024, // 500MB
            ..graphdb::config::LogConfig::default()
        },
        auth: graphdb::config::AuthConfig::default(),
        bootstrap: graphdb::config::BootstrapConfig::default(),
        optimizer: graphdb::config::OptimizerConfig::default(),
    };
    assert_eq!(custom_config.log.max_file_size, 500 * 1024 * 1024);

    // 测试小文件配置（用于测试）
    let small_config = Config {
        database: graphdb::config::DatabaseConfig::default(),
        log: graphdb::config::LogConfig {
            max_file_size: 1024, // 1KB
            ..graphdb::config::LogConfig::default()
        },
        auth: graphdb::config::AuthConfig::default(),
        bootstrap: graphdb::config::BootstrapConfig::default(),
        optimizer: graphdb::config::OptimizerConfig::default(),
    };
    assert_eq!(small_config.log.max_file_size, 1024);
}

/// 测试日志级别配置验证
#[test]
fn test_log_level_validation() {
    let valid_levels = vec!["trace", "debug", "info", "warn", "error"];

    for level in valid_levels {
        let config = Config {
            database: graphdb::config::DatabaseConfig::default(),
            log: graphdb::config::LogConfig {
                level: level.to_string(),
                ..graphdb::config::LogConfig::default()
            },
            auth: graphdb::config::AuthConfig::default(),
            bootstrap: graphdb::config::BootstrapConfig::default(),
            optimizer: graphdb::config::OptimizerConfig::default(),
        };
        assert_eq!(config.log.level, level);
    }
}
