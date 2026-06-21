//! Vertex Table Schema Management
//!
//! Handles schema operations like adding, removing, and renaming properties.

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

        self.schema.properties.push(prop.clone());
        self.columns
            .add_column(prop.name, prop.data_type, prop.nullable);

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

        if index == self.schema.primary_key_index {
            return Err(crate::core::StorageError::not_supported(
                "Removing the primary key property is not supported".to_string(),
            ));
        }

        self.schema.properties.remove(index);
        if index < self.schema.primary_key_index {
            self.schema.primary_key_index -= 1;
        }

        self.columns.remove_column(prop_name)?;
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

        self.schema.properties[index].name = new_name.to_string();
        self.columns.rename_column(old_name, new_name.to_string())?;
        Ok(())
    }
}
