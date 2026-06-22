//! Vertex Table Schema Management
//!
//! Handles schema operations like adding, removing, and renaming properties.
//! Schema modifications invalidate the property index cache, which is rebuilt on-demand.

use crate::core::StorageResult;
use crate::storage::types::StoragePropertyDef;

use super::core::VertexTable;

impl VertexTable {
    pub fn add_property(&mut self, prop: StoragePropertyDef) -> StorageResult<()> {
        if !self.is_open {
            return Err(crate::core::StorageError::storage_not_open());
        }

        if self.columns.get_column(&prop.name).is_some() {
            return Err(crate::core::StorageError::column_already_exists(prop.name.clone()));
        }

        // Add to columns first (potentially failing operation)
        self.columns
            .add_column(prop.name.clone(), prop.data_type.clone(), prop.nullable);

        // Only modify schema if columns addition succeeded
        self.schema.properties.push(prop.clone());

        // Update cache with new property
        let idx = self.schema.properties.len() - 1;
        self.property_index_cache.insert(prop.name, idx);

        // Increment schema version on modification
        self.schema.increment_version();

        Ok(())
    }

    pub fn remove_property(&mut self, prop_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(crate::core::StorageError::storage_not_open());
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == prop_name)
            .ok_or_else(|| crate::core::StorageError::column_not_found(prop_name.to_string()))?;

        // GUARD: Prevent removal of primary key property
        if index == self.schema.primary_key_index {
            return Err(crate::core::StorageError::not_supported(
                "Removing the primary key property is not supported".to_string(),
            ));
        }

        // Remove from columns first (potentially failing operation)
        self.columns.remove_column(prop_name)?;

        // Only modify schema if columns removal succeeded
        self.schema.properties.remove(index);
        if index < self.schema.primary_key_index {
            self.schema.primary_key_index -= 1;
        }

        // Rebuild cache: remove deleted property and adjust indices
        self.property_index_cache.remove(prop_name);
        for (name, idx) in &mut self.property_index_cache {
            if *idx > index {
                *idx -= 1;
            }
        }

        // Increment schema version on modification
        self.schema.increment_version();

        Ok(())
    }

    pub fn rename_property(&mut self, old_name: &str, new_name: &str) -> StorageResult<()> {
        if !self.is_open {
            return Err(crate::core::StorageError::storage_not_open());
        }

        if self
            .schema
            .properties
            .iter()
            .any(|prop| prop.name == new_name)
        {
            return Err(crate::core::StorageError::column_already_exists(new_name.to_string()));
        }

        let index = self
            .schema
            .properties
            .iter()
            .position(|prop| prop.name == old_name)
            .ok_or_else(|| crate::core::StorageError::column_not_found(old_name.to_string()))?;

        // Rename in columns first (potentially failing operation)
        self.columns.rename_column(old_name, new_name.to_string())?;

        // Only modify schema if columns rename succeeded
        self.schema.properties[index].name = new_name.to_string();

        // Update cache: rename key, keep index
        if let Some(idx) = self.property_index_cache.remove(old_name) {
            self.property_index_cache.insert(new_name.to_string(), idx);
        }

        // Increment schema version on modification
        self.schema.increment_version();

        Ok(())
    }
}
