use crate::core::types::Index;
use crate::core::StorageError;
use crate::storage::metadata::IndexMetadataManager;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct InMemoryIndexMetadataManager {
    tag_indexes: Arc<RwLock<HashMap<(u64, String), Index>>>,
    edge_indexes: Arc<RwLock<HashMap<(u64, String), Index>>>,
}

impl std::fmt::Debug for InMemoryIndexMetadataManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InMemoryIndexMetadataManager")
            .field("tag_indexes_count", &self.tag_indexes.read().len())
            .field("edge_indexes_count", &self.edge_indexes.read().len())
            .finish()
    }
}

impl InMemoryIndexMetadataManager {
    pub fn new() -> Self {
        Self {
            tag_indexes: Arc::new(RwLock::new(HashMap::new())),
            edge_indexes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryIndexMetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

impl IndexMetadataManager for InMemoryIndexMetadataManager {
    fn create_tag_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError> {
        let mut indexes = self.tag_indexes.write();
        let key = (space_id, index.name.clone());
        if indexes.contains_key(&key) {
            return Ok(false);
        }
        let mut index_with_space_id = index.clone();
        index_with_space_id.space_id = space_id;
        indexes.insert(key, index_with_space_id);
        Ok(true)
    }

    fn drop_tag_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError> {
        let mut indexes = self.tag_indexes.write();
        let key = (space_id, index_name.to_string());
        Ok(indexes.remove(&key).is_some())
    }

    fn get_tag_index(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<Option<Index>, StorageError> {
        let indexes = self.tag_indexes.read();
        Ok(indexes.get(&(space_id, index_name.to_string())).cloned())
    }

    fn list_tag_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError> {
        let indexes = self.tag_indexes.read();
        Ok(indexes
            .iter()
            .filter(|((sid, _), _)| *sid == space_id)
            .map(|(_, index)| index.clone())
            .collect())
    }

    fn drop_tag_indexes_by_tag(&self, space_id: u64, tag_name: &str) -> Result<(), StorageError> {
        let mut indexes = self.tag_indexes.write();
        indexes.retain(|_, index| !(index.space_id == space_id && index.schema_name == tag_name));
        Ok(())
    }

    fn create_edge_index(&self, space_id: u64, index: &Index) -> Result<bool, StorageError> {
        let mut indexes = self.edge_indexes.write();
        let key = (space_id, index.name.clone());
        if indexes.contains_key(&key) {
            return Ok(false);
        }
        let mut index_with_space_id = index.clone();
        index_with_space_id.space_id = space_id;
        indexes.insert(key, index_with_space_id);
        Ok(true)
    }

    fn drop_edge_index(&self, space_id: u64, index_name: &str) -> Result<bool, StorageError> {
        let mut indexes = self.edge_indexes.write();
        let key = (space_id, index_name.to_string());
        Ok(indexes.remove(&key).is_some())
    }

    fn get_edge_index(
        &self,
        space_id: u64,
        index_name: &str,
    ) -> Result<Option<Index>, StorageError> {
        let indexes = self.edge_indexes.read();
        Ok(indexes.get(&(space_id, index_name.to_string())).cloned())
    }

    fn list_edge_indexes(&self, space_id: u64) -> Result<Vec<Index>, StorageError> {
        let indexes = self.edge_indexes.read();
        Ok(indexes
            .iter()
            .filter(|((sid, _), _)| *sid == space_id)
            .map(|(_, index)| index.clone())
            .collect())
    }

    fn drop_edge_indexes_by_type(
        &self,
        space_id: u64,
        edge_type: &str,
    ) -> Result<(), StorageError> {
        let mut indexes = self.edge_indexes.write();
        indexes.retain(|_, index| !(index.space_id == space_id && index.schema_name == edge_type));
        Ok(())
    }
}
