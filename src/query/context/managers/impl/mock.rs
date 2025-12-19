//! Mock实现 - 用于测试的模拟管理器

use super::super::*;
use std::collections::HashMap;

/// Mock SchemaManager
#[derive(Debug)]
pub struct MockSchemaManager {
    schemas: HashMap<String, Schema>,
}

impl MockSchemaManager {
    pub fn new() -> Self {
        let mut schemas = HashMap::new();

        // 添加一些默认Schema
        schemas.insert(
            "person".to_string(),
            Schema {
                name: "person".to_string(),
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("name".to_string(), "string".to_string());
                    fields.insert("age".to_string(), "int".to_string());
                    fields
                },
                is_vertex: true,
            },
        );

        schemas.insert(
            "knows".to_string(),
            Schema {
                name: "knows".to_string(),
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert("weight".to_string(), "double".to_string());
                    fields
                },
                is_vertex: false,
            },
        );

        Self { schemas }
    }
}

impl SchemaManager for MockSchemaManager {
    fn get_schema(&self, name: &str) -> Option<Schema> {
        self.schemas.get(name).cloned()
    }

    fn list_schemas(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    fn has_schema(&self, name: &str) -> bool {
        self.schemas.contains_key(name)
    }
}

/// Mock IndexManager
#[derive(Debug)]
pub struct MockIndexManager {
    indexes: HashMap<String, Index>,
}

impl MockIndexManager {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }
}

impl IndexManager for MockIndexManager {
    fn create_index(&mut self, name: String, schema: Schema) -> Result<(), IndexError> {
        self.indexes.insert(name, schema);
        Ok(())
    }

    fn drop_index(&mut self, name: &str) -> Result<(), IndexError> {
        self.indexes.remove(name);
        Ok(())
    }

    fn get_index(&self, name: &str) -> Option<Index> {
        self.indexes.get(name).cloned()
    }
}

/// Mock StorageClient
#[derive(Debug)]
pub struct MockStorageClient {
    connected: bool,
}

impl MockStorageClient {
    pub fn new() -> Self {
        Self { connected: true }
    }
}

impl StorageClient for MockStorageClient {
    fn execute(&self, operation: StorageOperation) -> Result<StorageResponse, String> {
        if !self.connected {
            return Err("存储客户端未连接".to_string());
        }

        match operation {
            StorageOperation::Read { table, key } => Ok(StorageResponse {
                success: true,
                data: Some(Value::String(format!("读取 {}:{}", table, key))),
                error_message: None,
            }),
            StorageOperation::Write { table, key, value } => Ok(StorageResponse {
                success: true,
                data: Some(Value::String(format!("写入 {}:{}", table, key))),
                error_message: None,
            }),
            StorageOperation::Delete { table, key } => Ok(StorageResponse {
                success: true,
                data: Some(Value::String(format!("删除 {}:{}", table, key))),
                error_message: None,
            }),
            StorageOperation::Scan { table, prefix } => Ok(StorageResponse {
                success: true,
                data: Some(Value::String(format!("扫描 {} 前缀: {}", table, prefix))),
                error_message: None,
            }),
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// Mock MetaClient
#[derive(Debug)]
pub struct MockMetaClient {
    spaces: HashMap<i32, SpaceInfo>,
    connected: bool,
}

impl MockMetaClient {
    pub fn new() -> Self {
        let mut spaces = HashMap::new();

        // 添加默认空间
        spaces.insert(
            1,
            SpaceInfo {
                space_id: 1,
                space_name: "default".to_string(),
                partition_num: 10,
                replica_factor: 3,
            },
        );

        Self {
            spaces,
            connected: true,
        }
    }
}

impl MetaClient for MockMetaClient {
    fn get_cluster_info(&self) -> Result<ClusterInfo, String> {
        if !self.connected {
            return Err("元数据客户端未连接".to_string());
        }

        Ok(ClusterInfo {
            cluster_id: "mock_cluster".to_string(),
            meta_servers: vec!["localhost:9559".to_string()],
            storage_servers: vec!["localhost:9779".to_string()],
        })
    }

    fn get_space_info(&self, space_id: i32) -> Result<SpaceInfo, String> {
        if !self.connected {
            return Err("元数据客户端未连接".to_string());
        }

        self.spaces
            .get(&space_id)
            .cloned()
            .ok_or_else(|| format!("空间 {} 不存在", space_id))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// 辅助函数：获取空间信息
pub fn get_space(meta_client: &dyn MetaClient, space_id: i32) -> Option<SpaceInfo> {
    meta_client.get_space_info(space_id).ok()
}
