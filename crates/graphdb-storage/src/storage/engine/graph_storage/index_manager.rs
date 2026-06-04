use crate::core::metadata::index_manager::IndexMetadataManager;
use crate::core::types::Index;
use crate::core::{StorageError, StorageResult, Value};
use crate::storage::index::{EdgeIndexOps, VertexIndexOps};

use super::context::GraphStorageContext;

pub(crate) fn create_tag_index(
    ctx: &GraphStorageContext,
    space: &str,
    index: &Index,
) -> StorageResult<bool> {
    let space_id = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
        .space_id;
    ctx.index_metadata_manager()
        .create_tag_index(space_id, index)?;
    Ok(true)
}

pub(crate) fn drop_tag_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    let dropped = ctx
        .index_metadata_manager()
        .drop_tag_index(space_id, index_name)?;
    if dropped {
        ctx
            .index_data_manager()
            .write()
            .clear_tag_index(space_id, index_name)?;
    }
    Ok(dropped)
}

pub(crate) fn get_tag_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
) -> StorageResult<Option<Index>> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    ctx.index_metadata_manager()
        .get_tag_index(space_id, index_name)
}

pub(crate) fn list_tag_indexes(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Vec<Index>> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    ctx.index_metadata_manager().list_tag_indexes(space_id)
}

pub(crate) fn create_edge_index(
    ctx: &GraphStorageContext,
    space: &str,
    index: &Index,
) -> StorageResult<bool> {
    let space_id = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?
        .space_id;
    ctx.index_metadata_manager()
        .create_edge_index(space_id, index)?;
    Ok(true)
}

pub(crate) fn drop_edge_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    let dropped = ctx
        .index_metadata_manager()
        .drop_edge_index(space_id, index_name)?;
    if dropped {
        ctx
            .index_data_manager()
            .write()
            .clear_edge_index(space_id, index_name)?;
    }
    Ok(dropped)
}

pub(crate) fn get_edge_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
) -> StorageResult<Option<Index>> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    ctx.index_metadata_manager()
        .get_edge_index(space_id, index_name)
}

pub(crate) fn list_edge_indexes(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Vec<Index>> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    ctx.index_metadata_manager().list_edge_indexes(space_id)
}

pub(crate) fn rebuild_tag_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
    vertices: &[crate::core::Vertex],
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    let index = ctx
        .index_metadata_manager()
        .get_tag_index(space_id, index_name)?
        .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

    let ts = ctx.get_write_timestamp();
    for vertex in vertices {
        let props: Vec<(String, Value)> = vertex
            .properties
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let vid_value = Value::from(vertex.vid);
        ctx
            .update_vertex_indexes_mvcc(space_id, &vid_value, &index.name, &props, ts)?;
    }

    Ok(true)
}

pub(crate) fn rebuild_edge_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
    edges: &[crate::core::Edge],
) -> StorageResult<bool> {
    let space_id = ctx.schema_manager().get_space_id(space)?;
    let index = ctx
        .index_metadata_manager()
        .get_edge_index(space_id, index_name)?
        .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

    let ts = ctx.get_write_timestamp();
    for edge in edges {
        let props: Vec<(String, Value)> = edge
            .props
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        let src_value = Value::from(edge.src);
        let dst_value = Value::from(edge.dst);
        ctx.update_edge_indexes_mvcc(
            space_id,
            &src_value,
            &dst_value,
            &index.name,
            &props,
            ts,
        )?;
    }

    Ok(true)
}

pub(crate) fn lookup_index(
    ctx: &GraphStorageContext,
    space: &str,
    index_name: &str,
    value: &Value,
) -> StorageResult<Vec<Value>> {
    let space_id = ctx.schema_manager().get_space_id(space)?;

    let index = ctx
        .index_metadata_manager()
        .get_tag_index(space_id, index_name)?
        .ok_or_else(|| StorageError::not_found(format!("Index {} not found", index_name)))?;

    let results = ctx
        .index_data_manager()
        .read()
        .lookup_tag_index(space_id, &index, value)?;
    Ok(results)
}
