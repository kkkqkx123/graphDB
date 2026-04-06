//! Configuration loader framework
//!
//! Provides traits and implementations for loading configuration from various sources.
//!
//! # Examples
//!
//! ```rust
//! use bm25_service::config::{ConfigLoader, EnvLoader, ConfigFormat};
//!
//! // Load from environment variables with custom prefix
//! let loader = EnvLoader::new("MYAPP_INDEX_");
//! let env_vars = loader.load().unwrap();
//! ```
//!
//! ```no_run
//! use bm25_service::config::{ConfigLoader, FileLoader};
//!
//! // Load from TOML file
//! let loader = FileLoader::new("config.toml");
//! let file_vars = loader.load().unwrap();
//! ```

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

/// Configuration loading error
#[derive(Debug)]
pub enum LoaderError {
    /// File not found
    FileNotFound(String),
    /// Parse error
    ParseError(String),
    /// IO error
    IoError(std::io::Error),
    /// TOML parse error
    TomlError(toml::de::Error),
    /// YAML parse error
    YamlError(serde_yaml::Error),
    /// JSON parse error
    JsonError(serde_json::Error),
}

impl fmt::Display for LoaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoaderError::FileNotFound(path) => write!(f, "File not found: {}", path),
            LoaderError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LoaderError::IoError(err) => write!(f, "IO error: {}", err),
            LoaderError::TomlError(err) => write!(f, "TOML error: {}", err),
            LoaderError::YamlError(err) => write!(f, "YAML error: {}", err),
            LoaderError::JsonError(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl Error for LoaderError {}

impl From<std::io::Error> for LoaderError {
    fn from(err: std::io::Error) -> Self {
        LoaderError::IoError(err)
    }
}

impl From<toml::de::Error> for LoaderError {
    fn from(err: toml::de::Error) -> Self {
        LoaderError::TomlError(err)
    }
}

impl From<serde_yaml::Error> for LoaderError {
    fn from(err: serde_yaml::Error) -> Self {
        LoaderError::YamlError(err)
    }
}

impl From<serde_json::Error> for LoaderError {
    fn from(err: serde_json::Error) -> Self {
        LoaderError::JsonError(err)
    }
}

/// Result type for loader operations
pub type LoaderResult<T> = Result<T, LoaderError>;

/// Configuration format
#[derive(Debug, Clone, Copy)]
pub enum ConfigFormat {
    /// TOML format
    Toml,
    /// YAML format
    Yaml,
    /// JSON format
    Json,
}

/// Configuration loader trait
///
/// Implement this trait to load configuration from different sources.
pub trait ConfigLoader {
    /// Load configuration as key-value pairs
    fn load(&self) -> LoaderResult<HashMap<String, String>>;
}

/// Environment variable loader
///
/// Loads configuration from environment variables with a specified prefix.
///
/// # Examples
///
/// ```rust
/// use bm25_service::config::{EnvLoader, ConfigLoader};
///
/// // Load environment variables with prefix "MYAPP_INDEX_"
/// let loader = EnvLoader::new("MYAPP_INDEX_");
/// let vars = loader.load().unwrap();
///
/// // Environment variables:
/// // MYAPP_INDEX_WRITER_MEMORY_BUDGET=100000000
/// // MYAPP_INDEX_WRITER_NUM_THREADS=4
/// ```
pub struct EnvLoader {
    prefix: String,
}

impl EnvLoader {
    /// Create a new environment variable loader with the specified prefix
    ///
    /// # Arguments
    ///
    /// * `prefix` - Environment variable prefix (e.g., "MYAPP_INDEX_")
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_string(),
        }
    }

    /// Get the prefix
    pub fn prefix(&self) -> &str {
        &self.prefix
    }
}

impl ConfigLoader for EnvLoader {
    fn load(&self) -> LoaderResult<HashMap<String, String>> {
        Ok(std::env::vars()
            .filter(|(k, _)| k.starts_with(&self.prefix))
            .map(|(k, v)| {
                // Remove prefix and convert to lowercase
                let key = k
                    .trim_start_matches(&self.prefix)
                    .to_lowercase();
                (key, v)
            })
            .collect())
    }
}

/// File configuration loader
///
/// Loads configuration from a file in TOML, YAML, or JSON format.
///
/// # Examples
///
/// ```no_run
/// use bm25_service::config::{ConfigLoader, FileLoader, ConfigFormat};
///
/// // Load from TOML file (auto-detected)
/// let loader = FileLoader::new("config.toml");
/// let vars = loader.load().unwrap();
///
/// // Load from YAML file with explicit format
/// let loader = FileLoader::new("config.yaml").format(ConfigFormat::Yaml);
/// let vars = loader.load().unwrap();
/// ```
pub struct FileLoader {
    path: String,
    format: ConfigFormat,
}

impl FileLoader {
    /// Create a new file loader with the specified path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    pub fn new(path: &str) -> Self {
        let format = Self::detect_format(path);
        Self {
            path: path.to_string(),
            format,
        }
    }

    /// Set the configuration format
    ///
    /// # Arguments
    ///
    /// * `format` - Configuration format (TOML, YAML, or JSON)
    pub fn format(mut self, format: ConfigFormat) -> Self {
        self.format = format;
        self
    }

    /// Detect format from file extension
    fn detect_format(path: &str) -> ConfigFormat {
        if path.ends_with(".yaml") || path.ends_with(".yml") {
            ConfigFormat::Yaml
        } else if path.ends_with(".json") {
            ConfigFormat::Json
        } else {
            ConfigFormat::Toml // Default to TOML
        }
    }
}

impl ConfigLoader for FileLoader {
    fn load(&self) -> LoaderResult<HashMap<String, String>> {
        let content = std::fs::read_to_string(&self.path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    LoaderError::FileNotFound(self.path.clone())
                } else {
                    LoaderError::IoError(e)
                }
            })?;

        match self.format {
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(&content)?;
                Ok(flatten_toml(&toml_value, ""))
            }
            ConfigFormat::Yaml => {
                let yaml_value: serde_yaml::Value = serde_yaml::from_str(&content)?;
                Ok(flatten_yaml(&yaml_value, ""))
            }
            ConfigFormat::Json => {
                let json_value: serde_json::Value = serde_json::from_str(&content)?;
                Ok(flatten_json(&json_value, ""))
            }
        }
    }
}

/// Flatten TOML value to key-value pairs
fn flatten_toml(value: &toml::Value, prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    match value {
        toml::Value::Table(table) => {
            for (key, val) in table {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                result.extend(flatten_toml(val, &new_prefix));
            }
        }
        toml::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        toml::Value::Integer(i) => {
            result.insert(prefix.to_string(), i.to_string());
        }
        toml::Value::Boolean(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        toml::Value::Float(f) => {
            result.insert(prefix.to_string(), f.to_string());
        }
        _ => {} // Ignore other types
    }

    result
}

/// Flatten YAML value to key-value pairs
fn flatten_yaml(value: &serde_yaml::Value, prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    match value {
        serde_yaml::Value::Mapping(map) => {
            for (key, val) in map {
                if let Some(key_str) = key.as_str() {
                    let new_prefix = if prefix.is_empty() {
                        key_str.to_string()
                    } else {
                        format!("{}.{}", prefix, key_str)
                    };
                    result.extend(flatten_yaml(val, &new_prefix));
                }
            }
        }
        serde_yaml::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        serde_yaml::Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        serde_yaml::Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        _ => {} // Ignore other types
    }

    result
}

/// Flatten JSON value to key-value pairs
fn flatten_json(value: &serde_json::Value, prefix: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                let new_prefix = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                result.extend(flatten_json(val, &new_prefix));
            }
        }
        serde_json::Value::String(s) => {
            result.insert(prefix.to_string(), s.clone());
        }
        serde_json::Value::Number(n) => {
            result.insert(prefix.to_string(), n.to_string());
        }
        serde_json::Value::Bool(b) => {
            result.insert(prefix.to_string(), b.to_string());
        }
        _ => {} // Ignore other types
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_loader_new() {
        let loader = EnvLoader::new("TEST_");
        assert_eq!(loader.prefix(), "TEST_");
    }

    #[test]
    fn test_env_loader_load() {
        // Set test environment variables
        std::env::set_var("TEST_WRITER_MEMORY_BUDGET", "100000000");
        std::env::set_var("TEST_WRITER_THREADS", "4");
        std::env::set_var("OTHER_VAR", "should_not_appear");

        let loader = EnvLoader::new("TEST_");
        let vars = loader.load().unwrap();

        assert_eq!(vars.get("writer_memory_budget"), Some(&"100000000".to_string()));
        assert_eq!(vars.get("writer_threads"), Some(&"4".to_string()));
        assert!(!vars.contains_key("other_var"));

        // Cleanup
        std::env::remove_var("TEST_WRITER_MEMORY_BUDGET");
        std::env::remove_var("TEST_WRITER_THREADS");
        std::env::remove_var("OTHER_VAR");
    }

    #[test]
    fn test_file_loader_detect_format() {
        assert!(matches!(
            FileLoader::new("config.toml").format,
            ConfigFormat::Toml
        ));
        assert!(matches!(
            FileLoader::new("config.yaml").format,
            ConfigFormat::Yaml
        ));
        assert!(matches!(
            FileLoader::new("config.yml").format,
            ConfigFormat::Yaml
        ));
        assert!(matches!(
            FileLoader::new("config.json").format,
            ConfigFormat::Json
        ));
    }

    #[test]
    fn test_file_loader_format_override() {
        let loader = FileLoader::new("config.toml").format(ConfigFormat::Yaml);
        assert!(matches!(loader.format, ConfigFormat::Yaml));
    }

    #[test]
    fn test_flatten_toml() {
        let toml_str = r#"
            writer_memory_budget = 100000000
            
            [log_merge_policy]
            min_num_segments = 8
            max_docs_before_merge = 10000000
        "#;
        let toml_value: toml::Value = toml::from_str(toml_str).unwrap();
        let result = flatten_toml(&toml_value, "");

        assert_eq!(result.get("writer_memory_budget"), Some(&"100000000".to_string()));
        assert_eq!(result.get("log_merge_policy.min_num_segments"), Some(&"8".to_string()));
        assert_eq!(result.get("log_merge_policy.max_docs_before_merge"), Some(&"10000000".to_string()));
    }

    #[test]
    fn test_flatten_yaml() {
        let yaml_str = r#"
            writer_memory_budget: 100000000
            log_merge_policy:
              min_num_segments: 8
        "#;
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let result = flatten_yaml(&yaml_value, "");

        assert_eq!(result.get("writer_memory_budget"), Some(&"100000000".to_string()));
        assert_eq!(result.get("log_merge_policy.min_num_segments"), Some(&"8".to_string()));
    }

    #[test]
    fn test_flatten_json() {
        let json_str = r#"
            {
                "writer_memory_budget": 100000000,
                "log_merge_policy": {
                    "min_num_segments": 8
                }
            }
        "#;
        let json_value: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let result = flatten_json(&json_value, "");

        assert_eq!(result.get("writer_memory_budget"), Some(&"100000000".to_string()));
        assert_eq!(result.get("log_merge_policy.min_num_segments"), Some(&"8".to_string()));
    }
}
