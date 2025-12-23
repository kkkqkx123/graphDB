//! 元数据客户端实现 - 内存中的元数据管理

use super::super::{ClusterInfo, MetaClient, SpaceInfo};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 内存中的元数据客户端实现
#[derive(Debug, Clone)]
pub struct MemoryMetaClient {
    cluster_info: Arc<RwLock<ClusterInfo>>,
    spaces: Arc<RwLock<HashMap<i32, SpaceInfo>>>,
    connected: bool,
}

impl MemoryMetaClient {
    /// 创建新的内存元数据客户端
    pub fn new() -> Self {
        Self {
            cluster_info: Arc::new(RwLock::new(ClusterInfo {
                cluster_id: "local_cluster".to_string(),
                meta_servers: vec!["localhost:9559".to_string()],
                storage_servers: vec!["localhost:9779".to_string()],
            })),
            spaces: Arc::new(RwLock::new(HashMap::new())),
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

    /// 添加空间信息
    pub fn add_space(&self, space_info: SpaceInfo) -> Result<(), String> {
        let mut spaces = self.spaces.write().map_err(|e| e.to_string())?;
        spaces.insert(space_info.space_id, space_info);
        Ok(())
    }

    /// 删除空间信息
    pub fn remove_space(&self, space_id: i32) -> Result<(), String> {
        let mut spaces = self.spaces.write().map_err(|e| e.to_string())?;
        spaces.remove(&space_id);
        Ok(())
    }

    /// 更新空间信息
    pub fn update_space(&self, space_id: i32, space_info: SpaceInfo) -> Result<(), String> {
        let mut spaces = self.spaces.write().map_err(|e| e.to_string())?;
        spaces.insert(space_id, space_info);
        Ok(())
    }

    /// 列出所有空间ID
    pub fn list_space_ids(&self) -> Vec<i32> {
        match self.spaces.read() {
            Ok(spaces) => spaces.keys().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 检查空间是否存在
    pub fn has_space(&self, space_id: i32) -> bool {
        match self.spaces.read() {
            Ok(spaces) => spaces.contains_key(&space_id),
            Err(_) => false,
        }
    }

    /// 更新集群信息
    pub fn update_cluster_info(&self, cluster_info: ClusterInfo) -> Result<(), String> {
        let mut info = self.cluster_info.write().map_err(|e| e.to_string())?;
        *info = cluster_info;
        Ok(())
    }
}

impl Default for MemoryMetaClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MetaClient for MemoryMetaClient {
    fn get_cluster_info(&self) -> Result<ClusterInfo, String> {
        if !self.connected {
            return Err("元数据客户端未连接".to_string());
        }

        let info = self.cluster_info.read().map_err(|e| e.to_string())?;
        Ok(info.clone())
    }

    fn get_space_info(&self, space_id: i32) -> Result<SpaceInfo, String> {
        if !self.connected {
            return Err("元数据客户端未连接".to_string());
        }

        let spaces = self.spaces.read().map_err(|e| e.to_string())?;
        spaces
            .get(&space_id)
            .cloned()
            .ok_or_else(|| format!("空间 {} 不存在", space_id))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_meta_client_creation() {
        let client = MemoryMetaClient::new();
        assert!(client.is_connected());
        assert!(client.list_space_ids().is_empty());
    }

    #[test]
    fn test_memory_meta_client_get_cluster_info() {
        let client = MemoryMetaClient::new();

        let cluster_info = client.get_cluster_info();
        assert!(cluster_info.is_ok());

        let info = cluster_info.expect("Failed to get cluster info");
        assert_eq!(info.cluster_id, "local_cluster");
        assert_eq!(info.meta_servers.len(), 1);
        assert_eq!(info.storage_servers.len(), 1);
    }

    #[test]
    fn test_memory_meta_client_add_space() {
        let client = MemoryMetaClient::new();

        let space_info = SpaceInfo {
            space_id: 1,
            space_name: "test_space".to_string(),
            partition_num: 10,
            replica_factor: 3,
        };

        assert!(client.add_space(space_info.clone()).is_ok());
        assert!(client.has_space(1));
        assert_eq!(client.list_space_ids(), vec![1]);

        let retrieved = client.get_space_info(1);
        assert!(retrieved.is_ok());
        assert_eq!(
            retrieved.expect("Failed to retrieve space info").space_name,
            "test_space"
        );
    }

    #[test]
    fn test_memory_meta_client_remove_space() {
        let client = MemoryMetaClient::new();

        let space_info = SpaceInfo {
            space_id: 1,
            space_name: "test_space".to_string(),
            partition_num: 10,
            replica_factor: 3,
        };

        client.add_space(space_info).expect("Failed to add space");
        assert!(client.has_space(1));

        client.remove_space(1).expect("Failed to remove space");
        assert!(!client.has_space(1));
    }

    #[test]
    fn test_memory_meta_client_update_space() {
        let client = MemoryMetaClient::new();

        let space_info1 = SpaceInfo {
            space_id: 1,
            space_name: "old_name".to_string(),
            partition_num: 10,
            replica_factor: 3,
        };

        let space_info2 = SpaceInfo {
            space_id: 1,
            space_name: "new_name".to_string(),
            partition_num: 20,
            replica_factor: 5,
        };

        client.add_space(space_info1).expect("Failed to add space");

        let retrieved = client.get_space_info(1);
        assert!(retrieved.is_ok());
        assert_eq!(
            retrieved.expect("Failed to retrieve space info").space_name,
            "old_name"
        );

        client
            .update_space(1, space_info2)
            .expect("Failed to update space");

        let retrieved = client.get_space_info(1);
        assert!(retrieved.is_ok());
        assert_eq!(
            retrieved.expect("Failed to retrieve space info").space_name,
            "new_name"
        );
    }

    #[test]
    fn test_memory_meta_client_disconnect() {
        let mut client = MemoryMetaClient::new();
        assert!(client.is_connected());

        client.disconnect();
        assert!(!client.is_connected());

        let result = client.get_cluster_info();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "元数据客户端未连接");

        client.reconnect();
        assert!(client.is_connected());

        let result = client.get_cluster_info();
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_meta_client_update_cluster_info() {
        let client = MemoryMetaClient::new();

        let new_cluster_info = ClusterInfo {
            cluster_id: "new_cluster".to_string(),
            meta_servers: vec!["server1:9559".to_string(), "server2:9559".to_string()],
            storage_servers: vec!["server1:9779".to_string(), "server2:9779".to_string()],
        };

        assert!(client.update_cluster_info(new_cluster_info.clone()).is_ok());

        let retrieved = client.get_cluster_info();
        assert!(retrieved.is_ok());
        assert_eq!(
            retrieved
                .expect("Failed to retrieve cluster info")
                .cluster_id,
            "new_cluster"
        );
    }

    #[test]
    fn test_memory_meta_client_multiple_spaces() {
        let client = MemoryMetaClient::new();

        let space1 = SpaceInfo {
            space_id: 1,
            space_name: "space1".to_string(),
            partition_num: 10,
            replica_factor: 3,
        };

        let space2 = SpaceInfo {
            space_id: 2,
            space_name: "space2".to_string(),
            partition_num: 20,
            replica_factor: 5,
        };

        client.add_space(space1).expect("Failed to add space1");
        client.add_space(space2).expect("Failed to add space2");

        let space_ids = client.list_space_ids();
        assert_eq!(space_ids.len(), 2);
        assert!(space_ids.contains(&1));
        assert!(space_ids.contains(&2));

        assert!(client.has_space(1));
        assert!(client.has_space(2));
        assert!(!client.has_space(3));
    }
}
