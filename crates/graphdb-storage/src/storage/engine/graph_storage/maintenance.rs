use crate::core::{Edge, StorageError, StorageResult};
use crate::storage::StorageStats;

use super::context::GraphStorageContext;
use super::type_utils::edge_record_to_edge;
use super::writer;

pub(crate) fn get_storage_stats(ctx: &GraphStorageContext) -> StorageStats {
    let total_vertices = ctx.graph().total_vertex_count();
    let total_edges = ctx.graph().total_edge_count();

    let spaces = ctx.schema_manager().list_spaces().unwrap_or_default();
    let tags = spaces
        .iter()
        .filter_map(|s| ctx.schema_manager().list_tags(&s.space_name).ok())
        .flatten()
        .count();

    let edge_types = spaces
        .iter()
        .filter_map(|s| ctx.schema_manager().list_edge_types(&s.space_name).ok())
        .flatten()
        .count();

    let total_size = ctx.graph().storage_size() as u64;
    let data_size = ctx.graph().used_storage_size() as u64;

    StorageStats {
        total_vertices,
        total_edges,
        total_spaces: spaces.len(),
        total_tags: tags,
        total_edge_types: edge_types,
        total_size_bytes: total_size,
        data_size_bytes: data_size,
        index_size_bytes: total_size.saturating_sub(data_size),
    }
}

pub(crate) fn find_dangling_edges(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<Vec<Edge>> {
    let _space_info = ctx
        .schema_manager()
        .get_space(space)?
        .ok_or_else(|| StorageError::not_found(format!("Space {} not found", space)))?;

    let ts = ctx.get_read_timestamp();
    let mut dangling_edges = Vec::new();
    let edge_type_names: std::collections::HashMap<_, _> = ctx
        .schema_manager()
        .list_edge_types(space)?
        .into_iter()
        .map(|edge_type| (edge_type.edge_type_id, edge_type.edge_type_name))
        .collect();

    let edge_records = ctx.graph().collect_all_edge_records(ts);
    for (src_label_id, dst_label_id, edge_label_id, record) in edge_records {
        let Some(edge_type_name) = edge_type_names.get(&edge_label_id) else {
            continue;
        };
        let src_exists = ctx
            .graph()
            .get_vertex_by_internal_id(
                src_label_id,
                record.src_vid.as_int64().unwrap_or(0) as u32,
                ts,
            )
            .is_some();
        let dst_exists = ctx
            .graph()
            .get_vertex_by_internal_id(
                dst_label_id,
                record.dst_vid.as_int64().unwrap_or(0) as u32,
                ts,
            )
            .is_some();

        if !src_exists || !dst_exists {
            let edge = edge_record_to_edge(
                &record,
                edge_type_name,
                &format!("{}", record.src_vid),
                &format!("{}", record.dst_vid),
            );
            dangling_edges.push(edge);
        }
    }

    Ok(dangling_edges)
}

pub(crate) fn repair_dangling_edges(
    ctx: &GraphStorageContext,
    space: &str,
) -> StorageResult<usize> {
    let dangling_edges = find_dangling_edges(ctx, space)?;
    let mut repaired_count = 0;

    for edge in &dangling_edges {
        if writer::delete_edge(
            ctx,
            space,
            &edge.src,
            &edge.dst,
            &edge.edge_type,
            edge.ranking,
        )
        .is_ok()
        {
            repaired_count += 1;
        }
    }

    Ok(repaired_count)
}
