use std::sync::atomic::Ordering;

use crate::core::types::LabelId;
use crate::core::{StorageError, StorageResult};
use crate::storage::edge::{EdgeSchema, EdgeStrategy, EdgeTable};
use crate::storage::engine::data_store::EdgeTableKey;
use crate::storage::engine::params::CreateEdgeTypeParams;
use crate::storage::types::StoragePropertyDef;
use crate::storage::vertex::{VertexSchema, VertexTable};

use super::context::GraphStorageContext;

pub fn create_vertex_type(
    ctx: &GraphStorageContext,
    name: &str,
    properties: Vec<StoragePropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut vertex_label_names = ctx.data_store().vertex_label_names().write();
    if vertex_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }

    let mut vertex_label_counter = ctx.data_store().vertex_label_counter().write();
    let label_id = *vertex_label_counter;
    *vertex_label_counter += 1;

    let primary_key_index = properties
        .iter()
        .position(|p| p.name == primary_key)
        .ok_or_else(|| StorageError::property_not_found(primary_key.to_string()))?;

    let schema = VertexSchema {
        label_id,
        label_name: name.to_string(),
        properties,
        primary_key_index,
        schema_version: 1,
        schema_digest: String::new(),
    };

    let table = VertexTable::new(label_id, name.to_string(), schema);
    ctx.data_store()
        .vertex_tables()
        .write()
        .insert(label_id, table);
    vertex_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_vertex_type_with_id(
    ctx: &GraphStorageContext,
    name: &str,
    label_id: LabelId,
    properties: Vec<StoragePropertyDef>,
    primary_key: &str,
) -> StorageResult<LabelId> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut vertex_label_names = ctx.data_store().vertex_label_names().write();
    if vertex_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }

    if ctx
        .data_store()
        .vertex_tables()
        .read()
        .contains_key(&label_id)
    {
        return Err(StorageError::label_already_exists(format!(
            "label_id {}",
            label_id
        )));
    }

    let mut vertex_label_counter = ctx.data_store().vertex_label_counter().write();
    if label_id >= *vertex_label_counter {
        *vertex_label_counter = label_id + 1;
    }

    let primary_key_index = properties
        .iter()
        .position(|p| p.name == primary_key)
        .ok_or_else(|| StorageError::property_not_found(primary_key.to_string()))?;

    let schema = VertexSchema {
        label_id,
        label_name: name.to_string(),
        properties,
        primary_key_index,
        schema_version: 1,
        schema_digest: String::new(),
    };

    let table = VertexTable::new(label_id, name.to_string(), schema);
    ctx.data_store()
        .vertex_tables()
        .write()
        .insert(label_id, table);
    vertex_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_edge_type(
    ctx: &GraphStorageContext,
    name: &str,
    src_label: LabelId,
    dst_label: LabelId,
    properties: Vec<StoragePropertyDef>,
    oe_strategy: EdgeStrategy,
    ie_strategy: EdgeStrategy,
) -> StorageResult<LabelId> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    if !ctx
        .data_store()
        .vertex_tables()
        .read()
        .contains_key(&src_label)
    {
        return Err(StorageError::label_not_found(format!(
            "source label {}",
            src_label
        )));
    }
    if !ctx
        .data_store()
        .vertex_tables()
        .read()
        .contains_key(&dst_label)
    {
        return Err(StorageError::label_not_found(format!(
            "destination label {}",
            dst_label
        )));
    }

    let mut edge_label_names = ctx.data_store().edge_label_names().write();
    if edge_label_names.contains_key(name) {
        return Err(StorageError::label_already_exists(name.to_string()));
    }

    let mut edge_label_counter = ctx.data_store().edge_label_counter().write();
    let label_id = *edge_label_counter;
    *edge_label_counter += 1;

    let schema = EdgeSchema {
        label_id,
        label_name: name.to_string(),
        src_label,
        dst_label,
        properties,
        oe_strategy,
        ie_strategy,
    };

    let mut table = EdgeTable::new(schema)?;
    if let Some(stats) = ctx.stats_manager() {
        table.set_stats_manager(stats.clone());
    }
    let key = EdgeTableKey::new(src_label, dst_label, label_id);
    ctx.data_store().edge_tables().write().insert(key, table);
    edge_label_names.insert(name.to_string(), label_id);

    Ok(label_id)
}

pub fn create_edge_type_with_id(
    ctx: &GraphStorageContext,
    params: CreateEdgeTypeParams,
    label_id: LabelId,
) -> StorageResult<LabelId> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    if params.src_label != 0
        && !ctx
            .data_store()
            .vertex_tables()
            .read()
            .contains_key(&params.src_label)
    {
        return Err(StorageError::label_not_found(format!(
            "source label {}",
            params.src_label
        )));
    }
    if params.dst_label != 0
        && !ctx
            .data_store()
            .vertex_tables()
            .read()
            .contains_key(&params.dst_label)
    {
        return Err(StorageError::label_not_found(format!(
            "destination label {}",
            params.dst_label
        )));
    }

    let mut edge_label_names = ctx.data_store().edge_label_names().write();
    if edge_label_names.contains_key(params.name) {
        return Err(StorageError::label_already_exists(params.name.to_string()));
    }

    let mut edge_label_counter = ctx.data_store().edge_label_counter().write();
    if label_id >= *edge_label_counter {
        *edge_label_counter = label_id + 1;
    }

    let schema = EdgeSchema {
        label_id,
        label_name: params.name.to_string(),
        src_label: params.src_label,
        dst_label: params.dst_label,
        properties: params.properties,
        oe_strategy: params.oe_strategy,
        ie_strategy: params.ie_strategy,
    };

    let mut table = EdgeTable::new(schema)?;
    if let Some(stats) = ctx.stats_manager() {
        table.set_stats_manager(stats.clone());
    }
    let key = EdgeTableKey::new(params.src_label, params.dst_label, label_id);
    ctx.data_store().edge_tables().write().insert(key, table);
    edge_label_names.insert(params.name.to_string(), label_id);

    Ok(label_id)
}

pub fn drop_vertex_type(ctx: &GraphStorageContext, name: &str) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let label_id = {
        let mut vertex_label_names = ctx.data_store().vertex_label_names().write();
        vertex_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?
    };

    ctx.data_store().vertex_tables().write().remove(&label_id);
    ctx.data_store()
        .edge_tables()
        .write()
        .retain(|key, _table| key.src_label != label_id && key.dst_label != label_id);

    ctx.invalidate_vertex_cache(label_id);

    Ok(())
}

pub fn drop_edge_type(ctx: &GraphStorageContext, name: &str) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let label_id = {
        let mut edge_label_names = ctx.data_store().edge_label_names().write();
        edge_label_names
            .remove(name)
            .ok_or_else(|| StorageError::label_not_found(name.to_string()))?
    };

    ctx.data_store()
        .edge_tables()
        .write()
        .retain(|_key, _table| _key.edge_label != label_id);

    Ok(())
}

pub fn add_vertex_property(
    ctx: &GraphStorageContext,
    label: LabelId,
    prop: crate::storage::types::StoragePropertyDef,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut vertex_tables = ctx.data_store().vertex_tables().write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    table.add_property(prop)?;

    Ok(())
}

pub fn delete_vertex_property(
    ctx: &GraphStorageContext,
    label: LabelId,
    prop_name: &str,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut vertex_tables = ctx.data_store().vertex_tables().write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    table.remove_property(prop_name)
}

pub fn rename_vertex_property(
    ctx: &GraphStorageContext,
    label: LabelId,
    old_name: &str,
    new_name: &str,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut vertex_tables = ctx.data_store().vertex_tables().write();
    let table = vertex_tables
        .get_mut(&label)
        .ok_or_else(|| StorageError::label_not_found(format!("vertex label {}", label)))?;

    table.rename_property(old_name, new_name)
}

pub fn add_edge_property(
    ctx: &GraphStorageContext,
    edge_label: LabelId,
    prop: StoragePropertyDef,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut edge_tables = ctx.data_store().edge_tables().write();
    let mut updated = false;

    for (key, table) in edge_tables.iter_mut() {
        if key.edge_label == edge_label {
            table.add_property(prop.name.clone(), prop.data_type.clone(), prop.nullable)?;
            updated = true;
        }
    }

    if !updated {
        return Err(StorageError::label_not_found(format!(
            "edge label {}",
            edge_label
        )));
    }

    Ok(())
}

pub fn delete_edge_property(
    ctx: &GraphStorageContext,
    edge_label: LabelId,
    prop_name: &str,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut edge_tables = ctx.data_store().edge_tables().write();
    let mut updated = false;

    for table in edge_tables.values_mut() {
        if table.label() == edge_label {
            table.remove_property(prop_name)?;
            updated = true;
        }
    }

    if !updated {
        return Err(StorageError::label_not_found(format!(
            "edge label {}",
            edge_label
        )));
    }

    Ok(())
}

pub fn rename_edge_property(
    ctx: &GraphStorageContext,
    edge_label: LabelId,
    old_name: &str,
    new_name: &str,
) -> StorageResult<()> {
    if !ctx.is_open_flag().load(Ordering::Acquire) {
        return Err(StorageError::storage_not_open());
    }

    let mut edge_tables = ctx.data_store().edge_tables().write();
    let mut updated = false;

    for table in edge_tables.values_mut() {
        if table.label() == edge_label {
            table.rename_property(old_name, new_name)?;
            updated = true;
        }
    }

    if !updated {
        return Err(StorageError::label_not_found(format!(
            "edge label {}",
            edge_label
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::core::DataType;
    use crate::storage::edge::EdgeStrategy;
    use crate::storage::types::StoragePropertyDef;

    use super::super::GraphStorageContext;

    #[test]
    fn test_create_vertex_type() {
        let ctx = GraphStorageContext::new();
        let props = vec![StoragePropertyDef::new(
            "name".to_string(),
            DataType::String,
        )];
        let label_id = ctx
            .create_vertex_type("Person", props, "name")
            .expect("create_vertex_type should succeed");
        assert_eq!(label_id, 0);
    }

    #[test]
    fn test_create_duplicate_vertex_type() {
        let ctx = GraphStorageContext::new();
        let props = vec![StoragePropertyDef::new(
            "name".to_string(),
            DataType::String,
        )];
        ctx.create_vertex_type("Person", props, "name")
            .expect("create_vertex_type should succeed");
        let props2 = vec![StoragePropertyDef::new(
            "name".to_string(),
            DataType::String,
        )];
        let result = ctx.create_vertex_type("Person", props2, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_create_vertex_type_missing_primary_key() {
        let ctx = GraphStorageContext::new();
        let result = ctx.create_vertex_type(
            "Person",
            vec![StoragePropertyDef::new(
                "name".to_string(),
                DataType::String,
            )],
            "nonexistent",
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_create_edge_type() {
        let ctx = GraphStorageContext::new();
        let props = vec![StoragePropertyDef::new(
            "name".to_string(),
            DataType::String,
        )];
        ctx.create_vertex_type("Person", props, "name")
            .expect("create_vertex_type should succeed");

        let edge_label_id = ctx
            .create_edge_type(
                "KNOWS",
                0,
                0,
                vec![StoragePropertyDef::new("since".to_string(), DataType::Int)],
                EdgeStrategy::Multiple,
                EdgeStrategy::Multiple,
            )
            .expect("create_edge_type should succeed");
        assert_eq!(edge_label_id, 0);
    }

    #[test]
    fn test_create_edge_type_missing_src_label() {
        let ctx = GraphStorageContext::new();
        let result = ctx.create_edge_type(
            "KNOWS",
            0,
            0,
            vec![],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        );
        assert!(result.is_err());
    }

    fn name_prop() -> Vec<StoragePropertyDef> {
        vec![StoragePropertyDef::new(
            "name".to_string(),
            DataType::String,
        )]
    }

    #[test]
    fn test_drop_vertex_type() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        assert!(ctx
            .data_store()
            .vertex_label_names()
            .read()
            .contains_key("Person"));
        ctx.drop_vertex_type("Person").expect("drop should succeed");
        assert!(!ctx
            .data_store()
            .vertex_label_names()
            .read()
            .contains_key("Person"));
    }

    #[test]
    fn test_drop_nonexistent_vertex_type() {
        let ctx = GraphStorageContext::new();
        let result = ctx.drop_vertex_type("NonExistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_drop_edge_type() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        ctx.create_edge_type(
            "KNOWS",
            0,
            0,
            vec![],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .expect("create_edge_type should succeed");
        ctx.drop_edge_type("KNOWS").expect("drop should succeed");
    }

    #[test]
    fn test_add_vertex_property() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        ctx.add_vertex_property(
            0,
            StoragePropertyDef::new("email".to_string(), DataType::String),
        )
        .expect("add_vertex_property should succeed");
    }

    #[test]
    fn test_add_edge_property() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        ctx.create_edge_type(
            "KNOWS",
            0,
            0,
            vec![],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .expect("create_edge_type should succeed");
        ctx.add_edge_property(
            0,
            StoragePropertyDef::new("weight".to_string(), DataType::Double),
        )
        .expect("add_edge_property should succeed");
    }

    #[test]
    fn test_delete_vertex_property() {
        let ctx = GraphStorageContext::new();
        let props = vec![
            StoragePropertyDef::new("name".to_string(), DataType::String),
            StoragePropertyDef::new("age".to_string(), DataType::BigInt),
        ];
        ctx.create_vertex_type("Person", props, "name")
            .expect("create_vertex_type should succeed");
        ctx.delete_vertex_property(0, "age")
            .expect("delete_vertex_property should succeed");
    }

    #[test]
    fn test_rename_vertex_property() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        ctx.rename_vertex_property(0, "name", "full_name")
            .expect("rename_vertex_property should succeed");
    }

    #[test]
    fn test_delete_edge_property() {
        let ctx = GraphStorageContext::new();
        ctx.create_vertex_type("Person", name_prop(), "name")
            .expect("create_vertex_type should succeed");
        ctx.create_edge_type(
            "KNOWS",
            0,
            0,
            vec![StoragePropertyDef::new("since".to_string(), DataType::Int)],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .expect("create_edge_type should succeed");
        ctx.delete_edge_property(0, "since")
            .expect("delete_edge_property should succeed");
    }

    #[test]
    fn test_create_vertex_type_with_id() {
        let ctx = GraphStorageContext::new();
        let label_id = ctx
            .create_vertex_type_with_id("Person", 42, name_prop(), "name")
            .expect("create_vertex_type_with_id should succeed");
        assert_eq!(label_id, 42);
    }

    #[test]
    fn test_add_vertex_property_label_not_found() {
        let ctx = GraphStorageContext::new();
        let result = ctx.add_vertex_property(
            999,
            StoragePropertyDef::new("email".to_string(), DataType::String),
        );
        assert!(result.is_err());
    }
}
