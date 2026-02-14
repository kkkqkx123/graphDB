use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
pub mod test_config;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub storage_path: String,
    pub max_connections: usize,
    pub transaction_timeout: u64,
    pub log_level: String,
    pub log_dir: String,
    pub log_file: String,
    pub max_log_file_size: u64,
    pub max_log_files: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 9758,
            storage_path: "data/graphdb".to_string(),
            max_connections: 10,
            transaction_timeout: 30,
            log_level: "info".to_string(),
            log_dir: "logs".to_string(),
            log_file: "graphdb".to_string(),
            max_log_file_size: 100 * 1024 * 1024, // 100MB
            max_log_files: 5,
        }
    }
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.storage_path = Config::resolve_storage_path(&config.storage_path)?;
        Ok(config)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

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
