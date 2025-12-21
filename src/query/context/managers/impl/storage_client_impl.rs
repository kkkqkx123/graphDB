//! 存储客户端实现 - 内存中的存储操作

use super::super::{StorageClient, StorageOperation, StorageResponse};
use crate::core::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 内存中的存储客户端实现
#[derive(Debug, Clone)]
pub struct MemoryStorageClient {
    tables: Arc<RwLock<HashMap<String, HashMap<String, Value>>>>,
    connected: bool,
}

impl MemoryStorageClient {
    /// 创建新的内存存储客户端
    pub fn new() -> Self {
        Self {
            tables: Arc::new(RwLock::new(HashMap::new())),
            connected: true,
        }
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        self.connected = false;
    }

    /// 重新连接
    pub fn reconnect(&mut self) {
        self.connected = true;
    }

    /// 获取表数据
    pub fn get_table(&self, table_name: &str) -> Option<HashMap<String, Value>> {
        let tables = self.tables.read().ok()?;
        tables.get(table_name).cloned()
    }

    /// 列出所有表名
    pub fn list_tables(&self) -> Vec<String> {
        match self.tables.read() {
            Ok(tables) => tables.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 检查表是否存在
    pub fn has_table(&self, table_name: &str) -> bool {
        match self.tables.read() {
            Ok(tables) => tables.contains_key(table_name),
            Err(_) => false,
        }
    }

    /// 创建表
    pub fn create_table(&self, table_name: &str) -> Result<(), String> {
        let mut tables = self.tables.write().map_err(|e| e.to_string())?;
        if tables.contains_key(table_name) {
            return Err(format!("表 {} 已存在", table_name));
        }
        tables.insert(table_name.to_string(), HashMap::new());
        Ok(())
    }

    /// 删除表
    pub fn drop_table(&self, table_name: &str) -> Result<(), String> {
        let mut tables = self.tables.write().map_err(|e| e.to_string())?;
        tables.remove(table_name);
        Ok(())
    }
}

impl Default for MemoryStorageClient {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageClient for MemoryStorageClient {
    fn execute(&self, operation: StorageOperation) -> Result<StorageResponse, String> {
        if !self.connected {
            return Ok(StorageResponse {
                success: false,
                data: None,
                error_message: Some("存储客户端未连接".to_string()),
            });
        }

        let mut tables = self.tables.write().map_err(|e| e.to_string())?;

        match operation {
            StorageOperation::Read { table, key } => {
                let table_data = tables
                    .get_mut(&table)
                    .ok_or_else(|| format!("表 {} 不存在", table))?;
                let data = table_data.get(&key).cloned();

                Ok(StorageResponse {
                    success: true,
                    data,
                    error_message: None,
                })
            }

            StorageOperation::Write { table, key, value } => {
                let table_data = tables.entry(table).or_insert_with(HashMap::new);
                table_data.insert(key, value);

                Ok(StorageResponse {
                    success: true,
                    data: None,
                    error_message: None,
                })
            }

            StorageOperation::Delete { table, key } => {
                if let Some(table_data) = tables.get_mut(&table) {
                    table_data.remove(&key);
                    Ok(StorageResponse {
                        success: true,
                        data: None,
                        error_message: None,
                    })
                } else {
                    Ok(StorageResponse {
                        success: false,
                        data: None,
                        error_message: Some(format!("表 {} 不存在", table)),
                    })
                }
            }

            StorageOperation::Scan { table, prefix } => {
                let table_data = tables
                    .get(&table)
                    .ok_or_else(|| format!("表 {} 不存在", table))?;

                let mut results = HashMap::new();
                for (key, value) in table_data {
                    if key.starts_with(&prefix) {
                        results.insert(key.clone(), value.clone());
                    }
                }

                Ok(StorageResponse {
                    success: true,
                    data: Some(Value::Map(results)),
                    error_message: None,
                })
            }
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_storage_client_creation() {
        let client = MemoryStorageClient::new();
        assert!(client.is_connected());
        assert!(client.list_tables().is_empty());
    }

    #[test]
    fn test_memory_storage_client_read_write() {
        let client = MemoryStorageClient::new();

        // 写入数据
        let write_op = StorageOperation::Write {
            table: "users".to_string(),
            key: "user1".to_string(),
            value: Value::String("Alice".to_string()),
        };

        let result = client.execute(write_op);
        assert!(result.is_ok());
        assert!(result.expect("Result should be successful").success);

        // 读取数据
        let read_op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(read_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);
        assert_eq!(response.data, Some(Value::String("Alice".to_string())));
    }

    #[test]
    fn test_memory_storage_client_delete() {
        let client = MemoryStorageClient::new();

        // 先写入数据
        let write_op = StorageOperation::Write {
            table: "users".to_string(),
            key: "user1".to_string(),
            value: Value::String("Alice".to_string()),
        };
        client.execute(write_op).expect("Write operation should succeed");

        // 删除数据
        let delete_op = StorageOperation::Delete {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(delete_op);
        assert!(result.is_ok());
        assert!(result.expect("Result should be successful").success);

        // 验证数据已删除
        let read_op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(read_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);
        assert!(response.data.is_none());
    }

    #[test]
    fn test_memory_storage_client_scan() {
        let client = MemoryStorageClient::new();

        // 写入多个数据
        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "user1".to_string(),
                value: Value::String("Alice".to_string()),
            })
            .expect("Write operation should succeed");

        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "user2".to_string(),
                value: Value::String("Bob".to_string()),
            })
            .expect("Write operation should succeed");

        client
            .execute(StorageOperation::Write {
                table: "users".to_string(),
                key: "admin1".to_string(),
                value: Value::String("Admin".to_string()),
            })
            .expect("Write operation should succeed");

        // 扫描以"user"开头的数据
        let scan_op = StorageOperation::Scan {
            table: "users".to_string(),
            prefix: "user".to_string(),
        };

        let result = client.execute(scan_op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(response.success);

        if let Some(Value::Map(data)) = response.data {
            assert_eq!(data.len(), 2); // 应该找到user1和user2
            assert!(data.contains_key("user1"));
            assert!(data.contains_key("user2"));
            assert!(!data.contains_key("admin1"));
        } else {
            panic!("预期返回Map类型的数据");
        }
    }

    #[test]
    fn test_memory_storage_client_disconnect() {
        let mut client = MemoryStorageClient::new();
        assert!(client.is_connected());

        client.disconnect();
        assert!(!client.is_connected());

        let op = StorageOperation::Read {
            table: "users".to_string(),
            key: "user1".to_string(),
        };

        let result = client.execute(op);
        assert!(result.is_ok());
        let response = result.expect("Result should be available");
        assert!(!response.success);
        assert!(response.error_message.is_some());

        client.reconnect();
        assert!(client.is_connected());
    }

    #[test]
    fn test_memory_storage_client_table_operations() {
        let client = MemoryStorageClient::new();

        // 创建表
        assert!(client.create_table("users").is_ok());
        assert!(client.has_table("users"));
        assert_eq!(client.list_tables(), vec!["users".to_string()]);

        // 删除表
        assert!(client.drop_table("users").is_ok());
        assert!(!client.has_table("users"));
        assert!(client.list_tables().is_empty());
    }
}
