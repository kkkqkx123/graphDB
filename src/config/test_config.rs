//! 测试配置模块
//!
//! 统一管理测试数据路径，避免在项目路径中出现过多文件
//! 此模块仅在测试时编译

#[cfg(test)]
use std::path::PathBuf;

/// 测试配置结构体
#[cfg(test)]
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// 测试数据根目录
    pub test_data_root: PathBuf,
}

#[cfg(test)]
impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_data_root: PathBuf::from("data/tests"),
        }
    }
}

#[cfg(test)]
impl TestConfig {
    /// 创建新的测试配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取测试数据库路径
    pub fn test_db_path(&self, db_name: &str) -> PathBuf {
        self.test_data_root.join(db_name)
    }

    /// 获取测试图路径
    pub fn test_graph_path(&self) -> PathBuf {
        self.test_data_root.join("test_graph")
    }

    /// 获取临时存储路径
    pub fn temp_storage_path(&self) -> PathBuf {
        self.test_data_root.join("temp_storage")
    }

    /// 确保测试目录存在
    pub fn ensure_test_dirs(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.test_data_root)?;
        Ok(())
    }

    /// 清理测试数据
    pub fn cleanup_test_data(&self) -> std::io::Result<()> {
        if self.test_data_root.exists() {
            std::fs::remove_dir_all(&self.test_data_root)?;
        }
        Ok(())
    }
}

/// 全局测试配置实例
#[cfg(test)]
pub fn test_config() -> TestConfig {
    TestConfig::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_config_creation() {
        let config = TestConfig::new();
        assert!(config.test_data_root.ends_with("data/tests"));
    }

    #[test]
    fn test_test_db_path() {
        let config = TestConfig::new();
        let path = config.test_db_path("test_db");
        assert!(path.ends_with("data/tests/test_db"));
    }

    #[test]
    fn test_test_graph_path() {
        let config = TestConfig::new();
        let path = config.test_graph_path();
        assert!(path.ends_with("data/tests/test_graph"));
    }

    #[test]
    fn test_temp_storage_path() {
        let config = TestConfig::new();
        let path = config.temp_storage_path();
        assert!(path.ends_with("data/tests/temp_storage"));
    }
}
