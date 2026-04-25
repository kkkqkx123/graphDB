//! Configuration module tests
//!
//! Tests for configuration loading, validation, and building

use inversearch_service::config::{
    CacheConfig, Config, ConfigValidator, IndexConfig, LoggingConfig, ServerConfig, ValidationError,
};

/// Test Config default values
///
/// Verify default configuration values are valid
#[test]
fn test_config_defaults() {
    let config = Config::default();

    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 50051);
    assert_eq!(config.index.resolution, 9);
    assert!(!config.cache.enabled);
    assert!(!config.storage.enabled);
}

/// Test ServerConfig validation
///
/// Verify server configuration validation
#[test]
fn test_server_config_validation() {
    // Valid config
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 50051,
        workers: 4,
    };
    assert!(config.validate().is_ok());

    // Invalid: port 0
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 0,
        workers: 4,
    };
    assert!(config.validate().is_err());

    // Invalid: empty host
    let config = ServerConfig {
        host: "".to_string(),
        port: 50051,
        workers: 4,
    };
    assert!(config.validate().is_err());

    // Invalid: workers 0
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 50051,
        workers: 0,
    };
    assert!(config.validate().is_err());
}

/// Test IndexConfig validation
///
/// Verify index configuration validation
#[test]
fn test_index_config_validation() {
    // Valid config
    let config = IndexConfig {
        resolution: 9,
        tokenize: "strict".to_string(),
        depth: 0,
        bidirectional: true,
        fastupdate: false,
        keystore: None,
    };
    assert!(config.validate().is_ok());

    // Invalid: resolution too low
    let config = IndexConfig {
        resolution: 0,
        tokenize: "strict".to_string(),
        depth: 0,
        bidirectional: true,
        fastupdate: false,
        keystore: None,
    };
    assert!(config.validate().is_err());

    // Invalid: resolution too high
    let config = IndexConfig {
        resolution: 13,
        tokenize: "strict".to_string(),
        depth: 0,
        bidirectional: true,
        fastupdate: false,
        keystore: None,
    };
    assert!(config.validate().is_err());

    // Invalid: tokenize mode
    let config = IndexConfig {
        resolution: 9,
        tokenize: "invalid".to_string(),
        depth: 0,
        bidirectional: true,
        fastupdate: false,
        keystore: None,
    };
    assert!(config.validate().is_err());

    // Invalid: depth too high
    let config = IndexConfig {
        resolution: 9,
        tokenize: "strict".to_string(),
        depth: 11,
        bidirectional: true,
        fastupdate: false,
        keystore: None,
    };
    assert!(config.validate().is_err());
}

/// Test CacheConfig validation
///
/// Verify cache configuration validation
#[test]
fn test_cache_config_validation() {
    // Valid: disabled cache
    let config = CacheConfig {
        enabled: false,
        size: 0,
        ttl: None,
    };
    assert!(config.validate().is_ok());

    // Valid: enabled cache with proper size
    let config = CacheConfig {
        enabled: true,
        size: 1000,
        ttl: Some(3600),
    };
    assert!(config.validate().is_ok());

    // Invalid: enabled cache with size 0
    let config = CacheConfig {
        enabled: true,
        size: 0,
        ttl: None,
    };
    assert!(config.validate().is_err());

    // Invalid: cache size too large
    let config = CacheConfig {
        enabled: true,
        size: 2_000_000,
        ttl: None,
    };
    assert!(config.validate().is_err());

    // Invalid: TTL too large
    let config = CacheConfig {
        enabled: true,
        size: 1000,
        ttl: Some(100000),
    };
    assert!(config.validate().is_err());
}

/// Test LoggingConfig validation
///
/// Verify logging configuration validation
#[test]
fn test_logging_config_validation() {
    // Valid config
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "json".to_string(),
    };
    assert!(config.validate().is_ok());

    // Invalid: log level
    let config = LoggingConfig {
        level: "invalid".to_string(),
        format: "json".to_string(),
    };
    assert!(config.validate().is_err());

    // Invalid: log format
    let config = LoggingConfig {
        level: "info".to_string(),
        format: "invalid".to_string(),
    };
    assert!(config.validate().is_err());
}

/// Test full Config validation
///
/// Verify complete configuration validation
#[test]
fn test_full_config_validation() {
    // Valid config
    let config = Config::default();
    assert!(config.validate().is_ok());

    // Invalid server config
    let mut config = Config::default();
    config.server.port = 0;
    assert!(config.validate().is_err());

    // Invalid index config
    let mut config = Config::default();
    config.index.resolution = 0;
    assert!(config.validate().is_err());
}

/// Test configuration validation error messages
///
/// Verify validation errors provide helpful messages
#[test]
fn test_validation_error_messages() {
    let config = ServerConfig {
        host: "".to_string(),
        port: 50051,
        workers: 4,
    };

    let result = config.validate();
    assert!(result.is_err());

    if let Err(ValidationError::InvalidValue {
        field,
        value,
        reason,
    }) = result
    {
        assert_eq!(field, "server.host");
        assert_eq!(value, "empty");
        assert!(!reason.is_empty());
    } else {
        panic!("Expected InvalidValue error");
    }
}

/// Test Config from TOML string
///
/// Verify configuration can be loaded from TOML string
#[test]
fn test_config_from_toml_string() {
    let toml_content = r#"
        [server]
        host = "0.0.0.0"
        port = 50051
        workers = 4
        
        [index]
        resolution = 9
        tokenize = "strict"
        depth = 0
        bidirectional = true
        fastupdate = false
        
        [cache]
        enabled = false
        size = 1000
        ttl = 3600
        
        [storage]
        enabled = true
        backend = "coldwarmcache"
        
        [storage.file]
        base_path = "./data"
        auto_save = true
        save_interval_secs = 60
        
        [logging]
        level = "info"
        format = "json"
    "#;

    let config: Config = toml::from_str(toml_content).unwrap();

    assert_eq!(config.server.host, "0.0.0.0");
    assert_eq!(config.server.port, 50051);
    assert_eq!(config.index.resolution, 9);
    assert!(config.storage.enabled);
    assert_eq!(config.logging.level, "info");
}

/// Test Config validation after loading from TOML
///
/// Verify loaded configuration passes validation
#[test]
fn test_config_from_toml_with_validation() {
    let toml_content = r#"
        [server]
        host = "0.0.0.0"
        port = 50051
        workers = 4
        
        [index]
        resolution = 9
        tokenize = "strict"
        depth = 0
        bidirectional = true
        fastupdate = false
        
        [cache]
        enabled = false
        size = 1000
        
        [storage]
        enabled = false
        backend = "coldwarmcache"
        
        [logging]
        level = "info"
        format = "json"
    "#;

    let config: Config = toml::from_str(toml_content).unwrap();
    assert!(config.validate().is_ok());
}

/// Test invalid TOML configuration rejection
///
/// Verify invalid configurations are rejected
#[test]
fn test_invalid_config_rejection() {
    let toml_content = r#"
        [server]
        host = "0.0.0.0"
        port = 0  # Invalid port
        workers = 4
        
        [index]
        resolution = 9
        tokenize = "strict"
        depth = 0
        bidirectional = true
        fastupdate = false
        
        [cache]
        enabled = false
        size = 1000
        
        [storage]
        enabled = false
        backend = "coldwarmcache"
        
        [logging]
        level = "info"
        format = "json"
    "#;

    let config: Config = toml::from_str(toml_content).unwrap();
    assert!(config.validate().is_err());
}
