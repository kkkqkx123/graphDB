//! 元数据客户端实现 - 内存中的元数据管理

use super::super::{ClusterInfo, MetaClient, SpaceInfo};
use crate::core::error::{ManagerError, ManagerResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// 内存中的元数据客户端实现
#[derive(Debug, Clone)]
pub struct MemoryMetaClient {
    cluster_info: Arc<RwLock<ClusterInfo>>,
    spaces: Arc<RwLock<HashMap<i32, SpaceInfo>>>,
    next_space_id: Arc<RwLock<i32>>,
    storage_path: PathBuf,
    connected: bool,
}

impl MemoryMetaClient {
    /// 创建新的内存元数据客户端
    pub fn new() -> Self {
        Self::with_storage_path("./data/meta")
    }

    /// 使用指定存储路径创建内存元数据客户端
    pub fn with_storage_path<P: AsRef<Path>>(storage_path: P) -> Self {
        Self {
            cluster_info: Arc::new(RwLock::new(ClusterInfo {
                cluster_id: "local_cluster".to_string(),
                meta_servers: vec!["localhost:9559".to_string()],
                storage_servers: vec!["localhost:9779".to_string()],
            })),
            spaces: Arc::new(RwLock::new(HashMap::new())),
            next_space_id: Arc::new(RwLock::new(1)),
            storage_path: storage_path.as_ref().to_path_buf(),
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
    pub fn add_space(&self, space_info: SpaceInfo) -> ManagerResult<()> {
        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        spaces.insert(space_info.space_id, space_info);
        Ok(())
    }

    /// 删除空间信息
    pub fn remove_space(&self, space_id: i32) -> ManagerResult<()> {
        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        spaces.remove(&space_id);
        Ok(())
    }

    /// 更新空间信息
    pub fn update_space(&self, space_id: i32, space_info: SpaceInfo) -> ManagerResult<()> {
        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
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
    pub fn update_cluster_info(&self, cluster_info: ClusterInfo) -> ManagerResult<()> {
        let mut info = self
            .cluster_info
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
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
    fn get_cluster_info(&self) -> ManagerResult<ClusterInfo> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let info = self
            .cluster_info
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        Ok(info.clone())
    }

    fn get_space_info(&self, space_id: i32) -> ManagerResult<SpaceInfo> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        spaces
            .get(&space_id)
            .cloned()
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn create_space(
        &self,
        space_name: &str,
        partition_num: i32,
        replica_factor: i32,
    ) -> ManagerResult<i32> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        if partition_num <= 0 {
            return Err(ManagerError::InvalidInput("分区数必须大于0".to_string()));
        }

        if replica_factor <= 0 {
            return Err(ManagerError::InvalidInput("副本因子必须大于0".to_string()));
        }

        let mut next_id = self
            .next_space_id
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        let space_id = *next_id;
        *next_id += 1;
        drop(next_id);

        let space_info = SpaceInfo {
            space_id,
            space_name: space_name.to_string(),
            partition_num,
            replica_factor,
        };

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        spaces.insert(space_id, space_info);
        drop(spaces);

        self.save_to_disk()?;
        Ok(space_id)
    }

    fn drop_space(&self, space_id: i32) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        if !spaces.contains_key(&space_id) {
            return Err(ManagerError::NotFound(format!("空间 {} 不存在", space_id)));
        }
        spaces.remove(&space_id);
        drop(spaces);

        self.save_to_disk()?;
        Ok(())
    }

    fn list_spaces(&self) -> ManagerResult<Vec<SpaceInfo>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        let space_list: Vec<SpaceInfo> = spaces.values().cloned().collect();
        Ok(space_list)
    }

    fn has_space(&self, space_id: i32) -> bool {
        match self.spaces.read() {
            Ok(spaces) => spaces.contains_key(&space_id),
            Err(_) => false,
        }
    }

    fn load_from_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            return Ok(());
        }

        let cluster_file = self.storage_path.join("cluster.json");
        let spaces_file = self.storage_path.join("spaces.json");

        if cluster_file.exists() {
            let content = fs::read_to_string(&cluster_file)
                .map_err(|e| ManagerError::Other(e.to_string()))?;
            let cluster_info: ClusterInfo = serde_json::from_str(&content)
                .map_err(|e| ManagerError::Other(format!("反序列化集群信息失败: {}", e)))?;
            let mut info = self
                .cluster_info
                .write()
                .map_err(|e| ManagerError::Other(e.to_string()))?;
            *info = cluster_info;
        }

        if spaces_file.exists() {
            let content =
                fs::read_to_string(&spaces_file).map_err(|e| ManagerError::Other(e.to_string()))?;
            let space_list: Vec<SpaceInfo> = serde_json::from_str(&content)
                .map_err(|e| ManagerError::Other(format!("反序列化空间信息失败: {}", e)))?;
            let mut spaces = self
                .spaces
                .write()
                .map_err(|e| ManagerError::Other(e.to_string()))?;
            spaces.clear();
            for space_info in space_list {
                let space_id = space_info.space_id;
                spaces.insert(space_id, space_info);
                let mut next_id = self
                    .next_space_id
                    .write()
                    .map_err(|e| ManagerError::Other(e.to_string()))?;
                if space_id >= *next_id {
                    *next_id = space_id + 1;
                }
            }
        }

        Ok(())
    }

    fn save_to_disk(&self) -> ManagerResult<()> {
        use std::fs;

        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|e| ManagerError::Other(e.to_string()))?;
        }

        let cluster_info = self
            .cluster_info
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        let cluster_content = serde_json::to_string_pretty(&*cluster_info)
            .map_err(|e| ManagerError::Other(format!("序列化集群信息失败: {}", e)))?;

        let cluster_file = self.storage_path.join("cluster.json");
        fs::write(&cluster_file, cluster_content)
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        let space_list: Vec<SpaceInfo> = spaces.values().cloned().collect();
        let spaces_content = serde_json::to_string_pretty(&space_list)
            .map_err(|e| ManagerError::Other(format!("序列化空间信息失败: {}", e)))?;

        let spaces_file = self.storage_path.join("spaces.json");
        fs::write(&spaces_file, spaces_content).map_err(|e| ManagerError::Other(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

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
        assert_eq!(
            result.unwrap_err(),
            ManagerError::ConnectionError("元数据客户端未连接".to_string())
        );

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

    #[test]
    fn test_create_space() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");
        assert_eq!(space_id, 1);
        assert!(client.has_space(space_id));

        let space_info = client
            .get_space_info(space_id)
            .expect("Failed to get space info");
        assert_eq!(space_info.space_name, "test_space");
        assert_eq!(space_info.partition_num, 10);
        assert_eq!(space_info.replica_factor, 3);
    }

    #[test]
    fn test_create_space_invalid_params() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let result = client.create_space("test", 0, 3);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ManagerError::InvalidInput("分区数必须大于0".to_string())
        );

        let result = client.create_space("test", 10, 0);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ManagerError::InvalidInput("副本因子必须大于0".to_string())
        );
    }

    #[test]
    fn test_drop_space() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");
        assert!(client.has_space(space_id));

        client.drop_space(space_id).expect("Failed to drop space");
        assert!(!client.has_space(space_id));
    }

    #[test]
    fn test_drop_space_not_exist() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let result = client.drop_space(999);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            ManagerError::NotFound("空间 999 不存在".to_string())
        );
    }

    #[test]
    fn test_list_spaces() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        client
            .create_space("space1", 10, 3)
            .expect("Failed to create space1");
        client
            .create_space("space2", 20, 5)
            .expect("Failed to create space2");

        let spaces = client.list_spaces().expect("Failed to list spaces");
        assert_eq!(spaces.len(), 2);
        assert!(spaces.iter().any(|s| s.space_name == "space1"));
        assert!(spaces.iter().any(|s| s.space_name == "space2"));
    }

    #[test]
    fn test_save_and_load_from_disk() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let client1 = MemoryMetaClient::with_storage_path(storage_path);

        client1
            .create_space("space1", 10, 3)
            .expect("Failed to create space1");
        client1
            .create_space("space2", 20, 5)
            .expect("Failed to create space2");

        let new_cluster_info = ClusterInfo {
            cluster_id: "test_cluster".to_string(),
            meta_servers: vec!["server1:9559".to_string()],
            storage_servers: vec!["server1:9779".to_string()],
        };
        client1
            .update_cluster_info(new_cluster_info)
            .expect("Failed to update cluster info");

        client1.save_to_disk().expect("Failed to save to disk");

        assert!(storage_path.join("cluster.json").exists());
        assert!(storage_path.join("spaces.json").exists());

        let client2 = MemoryMetaClient::with_storage_path(storage_path);
        client2.load_from_disk().expect("Failed to load from disk");

        let spaces = client2.list_spaces().expect("Failed to list spaces");
        assert_eq!(spaces.len(), 2);

        let cluster_info = client2
            .get_cluster_info()
            .expect("Failed to get cluster info");
        assert_eq!(cluster_info.cluster_id, "test_cluster");
    }

    #[test]
    fn test_load_from_disk_empty() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let result = client.load_from_disk();
        assert!(result.is_ok());

        let spaces = client.list_spaces().expect("Failed to list spaces");
        assert!(spaces.is_empty());
    }

    #[test]
    fn test_auto_save_on_create_and_drop() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let client1 = MemoryMetaClient::with_storage_path(storage_path);
        client1
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        assert!(storage_path.join("spaces.json").exists());

        let client2 = MemoryMetaClient::with_storage_path(storage_path);
        client2.load_from_disk().expect("Failed to load from disk");
        assert!(client2.has_space(1));

        client2.drop_space(1).expect("Failed to drop space");

        let client3 = MemoryMetaClient::with_storage_path(storage_path);
        client3.load_from_disk().expect("Failed to load from disk");
        assert!(!client3.has_space(1));
    }
}
