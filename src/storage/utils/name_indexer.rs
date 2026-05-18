//! Name to Index Mapper
//!
//! Provides a reusable utility for mapping string names to numeric indices (PropertyId).
//! This replaces the duplicated `name_to_index: HashMap<String, usize>` pattern
//! found in both PropertyTable and ColumnStore.
//!
//! Features:
//! - O(1) name-to-index lookup
//! - O(1) index-to-name lookup
//! - Supports adding new mappings dynamically
//! - Memory-efficient storage

use std::collections::HashMap;

use crate::storage::storage_types::PropertyId;

/// Maps string names to PropertyId and vice versa.
#[derive(Debug, Clone)]
pub struct NameIndexer {
    name_to_id: HashMap<String, PropertyId>,
    id_to_name: Vec<Option<String>>,
    next_id: u16,
}

impl NameIndexer {
    pub fn new() -> Self {
        Self {
            name_to_id: HashMap::new(),
            id_to_name: Vec::new(),
            next_id: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            name_to_id: HashMap::with_capacity(capacity),
            id_to_name: Vec::with_capacity(capacity),
            next_id: 0,
        }
    }

    /// Register a new name and return its PropertyId.
    /// Returns the existing PropertyId if the name is already registered.
    pub fn register(&mut self, name: String) -> PropertyId {
        if let Some(&id) = self.name_to_id.get(&name) {
            return id;
        }

        let id = PropertyId::new(self.next_id);
        self.next_id += 1;

        self.name_to_id.insert(name.clone(), id);

        if id.as_usize() >= self.id_to_name.len() {
            self.id_to_name.resize(id.as_usize() + 1, None);
        }
        self.id_to_name[id.as_usize()] = Some(name);

        id
    }

    /// Look up the PropertyId for a given name.
    #[inline]
    pub fn get_id(&self, name: &str) -> Option<PropertyId> {
        self.name_to_id.get(name).copied()
    }

    /// Look up the name for a given PropertyId.
    #[inline]
    pub fn get_name(&self, id: PropertyId) -> Option<&str> {
        self.id_to_name
            .get(id.as_usize())
            .and_then(|opt| opt.as_deref())
    }

    /// Check if a name is registered.
    #[inline]
    pub fn contains(&self, name: &str) -> bool {
        self.name_to_id.contains_key(name)
    }

    /// Get the number of registered names.
    #[inline]
    pub fn len(&self) -> usize {
        self.name_to_id.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.name_to_id.is_empty()
    }

    /// Get all registered names.
    pub fn names(&self) -> Vec<&str> {
        self.id_to_name
            .iter()
            .filter_map(|opt| opt.as_deref())
            .collect()
    }

    /// Get all registered PropertyIds.
    pub fn ids(&self) -> Vec<PropertyId> {
        (0..self.next_id).map(PropertyId::new).collect()
    }

    /// Get the next PropertyId that will be assigned.
    #[inline]
    pub fn next_id(&self) -> u16 {
        self.next_id
    }

    /// Clear all registered names.
    pub fn clear(&mut self) {
        self.name_to_id.clear();
        self.id_to_name.clear();
        self.next_id = 0;
    }

    /// Remove a name from the indexer.
    /// Returns true if the name was found and removed.
    pub fn remove(&mut self, name: &str) -> bool {
        if let Some(id) = self.name_to_id.remove(name) {
            if id.as_usize() < self.id_to_name.len() {
                self.id_to_name[id.as_usize()] = None;
            }
            true
        } else {
            false
        }
    }

    /// Estimate memory usage in bytes.
    pub fn memory_size(&self) -> usize {
        let mut total = std::mem::size_of::<Self>();

        total += self.name_to_id.capacity()
            * (std::mem::size_of::<String>() + std::mem::size_of::<PropertyId>());

        for name in self.name_to_id.keys() {
            total += name.capacity();
        }

        total += self.id_to_name.capacity() * std::mem::size_of::<Option<String>>();

        total
    }
}

impl Default for NameIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_lookup() {
        let mut indexer = NameIndexer::new();

        let id1 = indexer.register("weight".to_string());
        let id2 = indexer.register("since".to_string());

        assert_eq!(id1.as_u16(), 0);
        assert_eq!(id2.as_u16(), 1);

        assert_eq!(indexer.get_id("weight"), Some(id1));
        assert_eq!(indexer.get_id("since"), Some(id2));
        assert_eq!(indexer.get_name(id1), Some("weight"));
        assert_eq!(indexer.get_name(id2), Some("since"));
    }

    #[test]
    fn test_duplicate_register() {
        let mut indexer = NameIndexer::new();

        let id1 = indexer.register("weight".to_string());
        let id2 = indexer.register("weight".to_string());

        assert_eq!(id1, id2);
        assert_eq!(indexer.len(), 1);
    }

    #[test]
    fn test_nonexistent_name() {
        let indexer = NameIndexer::new();

        assert_eq!(indexer.get_id("nonexistent"), None);
        assert!(!indexer.contains("nonexistent"));
    }

    #[test]
    fn test_clear() {
        let mut indexer = NameIndexer::new();

        indexer.register("weight".to_string());
        indexer.register("since".to_string());

        assert_eq!(indexer.len(), 2);

        indexer.clear();

        assert_eq!(indexer.len(), 0);
        assert_eq!(indexer.get_id("weight"), None);
    }

    #[test]
    fn test_names_and_ids() {
        let mut indexer = NameIndexer::new();

        indexer.register("weight".to_string());
        indexer.register("since".to_string());

        let names = indexer.names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"weight"));
        assert!(names.contains(&"since"));

        let ids = indexer.ids();
        assert_eq!(ids.len(), 2);
        assert_eq!(ids[0].as_u16(), 0);
        assert_eq!(ids[1].as_u16(), 1);
    }
}
