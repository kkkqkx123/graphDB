use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[cfg(test)]
pub mod test_config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub storage_path: String,
    pub cache_size: usize,
    pub enable_cache: bool,
    pub max_connections: usize,
    pub transaction_timeout: u64, // in seconds
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),
            cache_size: 1000,
            enable_cache: true,
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
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
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 9758);
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
        assert_eq!(config.host, loaded_config.host);
        assert_eq!(config.port, loaded_config.port);
    }
}
