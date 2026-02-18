//! 集成测试共享工具模块
//!
//! 提供测试基础设施和辅助函数，供所有集成测试使用

pub mod assertions;
pub mod data_fixtures;
pub mod storage_helpers;

use std::sync::Arc;
use parking_lot::Mutex;
use std::path::PathBuf;
use graphdb::storage::redb_storage::RedbStorage;

/// 测试存储实例包装器
///
/// 使用项目目录下的临时文件夹确保每个测试有独立的存储环境，
/// 测试结束后自动清理临时目录
pub struct TestStorage {
    storage: Arc<Mutex<RedbStorage>>,
    temp_path: PathBuf,
}

impl TestStorage {
    /// 创建新的测试存储实例
    pub fn new() -> anyhow::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("test.db");
        
        let storage = Arc::new(Mutex::new(RedbStorage::new_with_path(db_path)?));
        Ok(Self {
            storage,
            temp_path: temp_dir.path().to_path_buf(),
        })
    }

    /// 获取存储实例引用
    pub fn storage(&self) -> Arc<Mutex<RedbStorage>> {
        self.storage.clone()
    }
}

impl Drop for TestStorage {
    fn drop(&mut self) {
        // 尝试清理临时目录，忽略错误
        let _ = std::fs::remove_dir_all(&self.temp_path);
    }
}

/// 测试上下文，包含常用测试资源
pub struct TestContext {
    pub storage: TestStorage,
}

impl TestContext {
    /// 创建新的测试上下文
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            storage: TestStorage::new()?,
        })
    }
}
