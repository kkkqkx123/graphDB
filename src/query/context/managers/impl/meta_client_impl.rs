//! 元数据客户端实现 - 内存中的元数据管理

use super::super::{
    ClusterInfo, EdgeTypeDef, MetaClient, MetadataVersion, SpaceInfo,
    TagDef,
};
use crate::query::context::managers::types::{PropertyDef as ManagerPropertyDef, PropertyType as ManagerPropertyType};
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
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        Self {
            cluster_info: Arc::new(RwLock::new(ClusterInfo {
                cluster_id: "local_cluster".to_string(),
                meta_servers: vec!["localhost:9559".to_string()],
                storage_servers: vec!["localhost:9779".to_string()],
                version: MetadataVersion {
                    version: 1,
                    timestamp: current_time,
                    description: "初始版本".to_string(),
                },
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

    /// 验证标签定义
    fn validate_tag_def(&self, tag_def: &TagDef) -> ManagerResult<()> {
        if tag_def.tag_name.is_empty() {
            return Err(ManagerError::InvalidInput("标签名称不能为空".to_string()));
        }

        let mut property_names = std::collections::HashSet::new();
        for prop in &tag_def.properties {
            if prop.name.is_empty() {
                return Err(ManagerError::InvalidInput("属性名称不能为空".to_string()));
            }
            if property_names.contains(&prop.name) {
                return Err(ManagerError::InvalidInput(format!(
                    "属性名称重复: {}",
                    prop.name
                )));
            }
            property_names.insert(prop.name.clone());
        }

        Ok(())
    }

    /// 验证边类型定义
    fn validate_edge_type_def(&self, edge_type_def: &EdgeTypeDef) -> ManagerResult<()> {
        if edge_type_def.edge_name.is_empty() {
            return Err(ManagerError::InvalidInput("边类型名称不能为空".to_string()));
        }

        let mut property_names = std::collections::HashSet::new();
        for prop in &edge_type_def.properties {
            if prop.name.is_empty() {
                return Err(ManagerError::InvalidInput("属性名称不能为空".to_string()));
            }
            if property_names.contains(&prop.name) {
                return Err(ManagerError::InvalidInput(format!(
                    "属性名称重复: {}",
                    prop.name
                )));
            }
            property_names.insert(prop.name.clone());
        }

        Ok(())
    }

    /// 创建元数据版本
    fn create_metadata_version(&self, description: &str) -> MetadataVersion {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        MetadataVersion {
            version: 1,
            timestamp: current_time,
            description: description.to_string(),
        }
    }

    /// 增加元数据版本
    fn increment_metadata_version(&self, current_version: &MetadataVersion, description: &str) -> MetadataVersion {
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64;

        MetadataVersion {
            version: current_version.version + 1,
            timestamp: current_time,
            description: description.to_string(),
        }
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
            tags: Vec::new(),
            edge_types: Vec::new(),
            version: self.create_metadata_version("创建空间"),
        };

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;
        spaces.insert(space_id, space_info);

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
        spaces
            .remove(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

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
        Ok(spaces.values().cloned().collect())
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

        let spaces_file = self.storage_path.join("spaces.json");
        if !spaces_file.exists() {
            return Ok(());
        }

        let spaces_content = fs::read_to_string(&spaces_file)
            .map_err(|e| ManagerError::Other(format!("读取空间文件失败: {}", e)))?;

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let loaded_spaces: Vec<SpaceInfo> = serde_json::from_str(&spaces_content)
            .map_err(|e| ManagerError::Other(format!("解析空间文件失败: {}", e)))?;

        for space in loaded_spaces {
            spaces.insert(space.space_id, space);
        }

        Ok(())
    }

    fn save_to_disk(&self) -> ManagerResult<()> {
        use std::fs;

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

    fn create_tag(&self, space_id: i32, tag_def: TagDef) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        self.validate_tag_def(&tag_def)?;

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get_mut(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        if space_info.tags.iter().any(|t| t.tag_name == tag_def.tag_name) {
            return Err(ManagerError::InvalidInput(format!(
                "标签 {} 已存在",
                tag_def.tag_name
            )));
        }

        space_info.tags.push(tag_def);
        drop(spaces);

        self.save_to_disk()?;
        Ok(())
    }

    fn drop_tag(&self, space_id: i32, tag_name: &str) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get_mut(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        let original_len = space_info.tags.len();
        space_info.tags.retain(|t| t.tag_name != tag_name);

        if space_info.tags.len() == original_len {
            return Err(ManagerError::NotFound(format!("标签 {} 不存在", tag_name)));
        }

        drop(spaces);

        self.save_to_disk()?;
        Ok(())
    }

    fn get_tag(&self, space_id: i32, tag_name: &str) -> ManagerResult<TagDef> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        space_info
            .tags
            .iter()
            .find(|t| t.tag_name == tag_name)
            .cloned()
            .ok_or_else(|| ManagerError::NotFound(format!("标签 {} 不存在", tag_name)))
    }

    fn list_tags(&self, space_id: i32) -> ManagerResult<Vec<TagDef>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        Ok(space_info.tags.clone())
    }

    fn create_edge_type(&self, space_id: i32, edge_type_def: EdgeTypeDef) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        self.validate_edge_type_def(&edge_type_def)?;

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get_mut(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        if space_info
            .edge_types
            .iter()
            .any(|e| e.edge_name == edge_type_def.edge_name)
        {
            return Err(ManagerError::InvalidInput(format!(
                "边类型 {} 已存在",
                edge_type_def.edge_name
            )));
        }

        space_info.edge_types.push(edge_type_def);
        drop(spaces);

        self.save_to_disk()?;
        Ok(())
    }

    fn drop_edge_type(&self, space_id: i32, edge_name: &str) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get_mut(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        let original_len = space_info.edge_types.len();
        space_info.edge_types.retain(|e| e.edge_name != edge_name);

        if space_info.edge_types.len() == original_len {
            return Err(ManagerError::NotFound(format!(
                "边类型 {} 不存在",
                edge_name
            )));
        }

        drop(spaces);

        self.save_to_disk()?;
        Ok(())
    }

    fn get_edge_type(&self, space_id: i32, edge_name: &str) -> ManagerResult<EdgeTypeDef> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        space_info
            .edge_types
            .iter()
            .find(|e| e.edge_name == edge_name)
            .cloned()
            .ok_or_else(|| ManagerError::NotFound(format!("边类型 {} 不存在", edge_name)))
    }

    fn list_edge_types(&self, space_id: i32) -> ManagerResult<Vec<EdgeTypeDef>> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        Ok(space_info.edge_types.clone())
    }

    fn get_metadata_version(&self, space_id: i32) -> ManagerResult<MetadataVersion> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let spaces = self
            .spaces
            .read()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        Ok(space_info.version.clone())
    }

    fn update_metadata_version(&self, space_id: i32, description: &str) -> ManagerResult<()> {
        if !self.connected {
            return Err(ManagerError::ConnectionError(
                "元数据客户端未连接".to_string(),
            ));
        }

        let mut spaces = self
            .spaces
            .write()
            .map_err(|e| ManagerError::Other(e.to_string()))?;

        let space_info = spaces
            .get_mut(&space_id)
            .ok_or_else(|| ManagerError::NotFound(format!("空间 {} 不存在", space_id)))?;

        space_info.version = self.increment_metadata_version(&space_info.version, description);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_meta_client_creation() {
        let client = MemoryMetaClient::new();
        assert!(client.is_connected());
        assert_eq!(client.list_space_ids(), Vec::<i32>::new());
    }

    #[test]
    fn test_create_space() {
        let temp_dir: tempfile::TempDir = tempdir().expect("Failed to create temp dir");
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
    fn test_list_spaces() {
        let temp_dir: tempfile::TempDir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        client
            .create_space("space1", 10, 3)
            .expect("Failed to create space1");
        client
            .create_space("space2", 5, 2)
            .expect("Failed to create space2");

        let mut spaces = client.list_spaces().expect("Failed to list spaces");
        assert_eq!(spaces.len(), 2);
        spaces.sort_by_key(|s| s.space_id);
        assert_eq!(spaces[0].space_name, "space1");
        assert_eq!(spaces[1].space_name, "space2");
    }

    #[test]
    fn test_drop_space() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        client
            .drop_space(space_id)
            .expect("Failed to drop space");

        assert!(!client.has_space(space_id));
    }

    #[test]
    fn test_create_tag() {
        let temp_dir: tempfile::TempDir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        let tag_def = TagDef {
            tag_name: "person".to_string(),
            properties: vec![
                ManagerPropertyDef {
                    name: "name".to_string(),
                    type_: ManagerPropertyType::String,
                    nullable: false,
                    default: None,
                },
                ManagerPropertyDef {
                    name: "age".to_string(),
                    type_: ManagerPropertyType::Int,
                    nullable: true,
                    default: None,
                },
            ],
        };

        client
            .create_tag(space_id, tag_def.clone())
            .expect("Failed to create tag");

        let tags = client.list_tags(space_id).expect("Failed to list tags");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].tag_name, "person");

        let retrieved = client
            .get_tag(space_id, "person")
            .expect("Failed to get tag");
        assert_eq!(retrieved.tag_name, "person");
        assert_eq!(retrieved.properties.len(), 2);
    }

    #[test]
    fn test_create_tag_invalid() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        let tag_def = TagDef {
            tag_name: "".to_string(),
            properties: vec![],
        };

        let result = client.create_tag(space_id, tag_def);
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_tag() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        let tag_def = TagDef {
            tag_name: "person".to_string(),
            properties: vec![],
        };

        client
            .create_tag(space_id, tag_def)
            .expect("Failed to create tag");

        client
            .drop_tag(space_id, "person")
            .expect("Failed to drop tag");

        let tags = client.list_tags(space_id).expect("Failed to list tags");
        assert!(tags.is_empty());
    }

    #[test]
    fn test_create_edge_type() {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let client = MemoryMetaClient::with_storage_path(temp_dir.path());

        let space_id = client
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        let edge_type_def = EdgeTypeDef {
            edge_name: "knows".to_string(),
            properties: vec![
                ManagerPropertyDef {
                    name: "since".to_string(),
                    type_: ManagerPropertyType::Int,
                    nullable: true,
                    default: None,
                },
            ],
        };

        client
            .create_edge_type(space_id, edge_type_def.clone())
            .expect("Failed to create edge type");

        let edge_types = client
            .list_edge_types(space_id)
            .expect("Failed to list edge types");
        assert_eq!(edge_types.len(), 1);
        assert_eq!(edge_types[0].edge_name, "knows");

        let retrieved = client
            .get_edge_type(space_id, "knows")
            .expect("Failed to get edge type");
        assert_eq!(retrieved.edge_name, "knows");
        assert_eq!(retrieved.properties.len(), 1);
    }

    #[test]
    fn test_persistence() {
        let temp_dir: tempfile::TempDir = tempdir().expect("Failed to create temp dir");
        let storage_path = temp_dir.path();

        let client1 = MemoryMetaClient::with_storage_path(storage_path);
        let space_id = client1
            .create_space("test_space", 10, 3)
            .expect("Failed to create space");

        let tag_def = TagDef {
            tag_name: "person".to_string(),
            properties: vec![],
        };
        client1
            .create_tag(space_id, tag_def)
            .expect("Failed to create tag");

        let edge_type_def = EdgeTypeDef {
            edge_name: "knows".to_string(),
            properties: vec![],
        };
        client1
            .create_edge_type(space_id, edge_type_def)
            .expect("Failed to create edge type");

        let client2 = MemoryMetaClient::with_storage_path(storage_path);
        client2.load_from_disk().expect("Failed to load from disk");

        let tags = client2.list_tags(space_id).expect("Failed to list tags");
        assert_eq!(tags.len(), 1);

        let edge_types = client2
            .list_edge_types(space_id)
            .expect("Failed to list edge types");
        assert_eq!(edge_types.len(), 1);
    }
}
