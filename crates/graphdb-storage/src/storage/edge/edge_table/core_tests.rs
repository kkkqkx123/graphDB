use super::*;
use crate::core::types::VertexId;
use crate::core::Value;

fn create_test_schema() -> EdgeSchema {
    EdgeSchema {
        label_id: 0,
        label_name: "knows".to_string(),
        src_label: 0,
        dst_label: 0,
        properties: vec![StoragePropertyDef::new(
            "weight".to_string(),
            DataType::Double,
        )],
        oe_strategy: EdgeStrategy::Multiple,
        ie_strategy: EdgeStrategy::Multiple,
    }
}

#[test]
fn test_insert_and_get() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    assert!(table.has_edge(0, 1, 0, 100));

    let edge = table.get_edge(0, 1, 0, 100).unwrap();
    assert_eq!(edge.src_vid, VertexId::from_int64(0));
    assert_eq!(edge.dst_vid, VertexId::from_int64(1));
    assert_eq!(edge.properties.len(), 1);
}

#[test]
fn test_rank_distinguishes_parallel_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 10, &[("weight".to_string(), Value::Double(1.0))], 100)
        .unwrap();
    table
        .insert_edge(0, 1, 20, &[("weight".to_string(), Value::Double(2.0))], 100)
        .unwrap();

    let rank_10 = table.get_edge(0, 1, 10, 100).unwrap();
    let rank_20 = table.get_edge(0, 1, 20, 100).unwrap();

    assert_eq!(rank_10.rank, 10);
    assert_eq!(rank_20.rank, 20);
    assert_eq!(table.out_edges(0, 100).len(), 2);
}

#[test]
fn test_delete() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    assert!(table.delete_edge(0, 1, 0, 200).unwrap());
    assert!(!table.has_edge(0, 1, 0, 300));
}

#[test]
fn test_out_in_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table.insert_edge(0, 1, 0, &[], 100).unwrap();
    table.insert_edge(0, 2, 0, &[], 100).unwrap();
    table.insert_edge(1, 0, 0, &[], 100).unwrap();

    assert_eq!(table.out_edges(0, 100).len(), 2);
    assert_eq!(table.in_edges(0, 100).len(), 1);
    assert_eq!(table.out_edges(1, 100).len(), 1);
    assert_eq!(table.in_edges(1, 100).len(), 1);
}

#[test]
fn test_update_edge_property() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.0))], 100)
        .unwrap();

    let updated = table
        .update_edge_property(0, 1, 0, "weight", &Value::Double(2.0), 100)
        .unwrap();
    assert!(updated);

    let edge = table.get_edge(0, 1, 0, 100).unwrap();
    assert_eq!(edge.properties.len(), 1);
}

#[test]
fn test_self_loop_edge() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 0, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    assert!(table.has_edge(0, 0, 0, 100));

    let out_edges = table.out_edges(0, 100);
    assert!(out_edges.iter().any(|edge| edge.dst_vid == VertexId::from_int64(0)));

    let in_edges = table.in_edges(0, 100);
    assert!(in_edges.iter().any(|edge| edge.src_vid == VertexId::from_int64(0)));
}

#[test]
fn test_multiple_parallel_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 1u32;
    let dst = 2u32;

    for rank in 0..5 {
        table
            .insert_edge(
                src,
                dst,
                rank as i64,
                &[("weight".to_string(), Value::Double((rank as f64) * 0.5))],
                100,
            )
            .unwrap();
    }

    for rank in 0..5 {
        assert!(table.has_edge(src, dst, rank as i64, 100));
    }

    let edges = table.out_edges(src, 100);
    assert_eq!(edges.len(), 5);

    let incoming = table.in_edges(dst, 100);
    assert_eq!(incoming.len(), 5);
}

#[test]
fn test_edge_deletion_with_timestamps() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    assert!(table.has_edge(0, 1, 0, 100));

    let deleted = table.delete_edge(0, 1, 0, 200).unwrap();
    assert!(deleted);

    assert!(!table.has_edge(0, 1, 0, 200));
    assert!(!table.has_edge(0, 1, 0, 300));
}

#[test]
fn test_property_updates_multiple_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    for i in 0..3 {
        table
            .insert_edge(0, 1, i as i64, &[("weight".to_string(), Value::Double(1.0))], 100)
            .unwrap();
    }

    for i in 0..3 {
        let updated = table
            .update_edge_property(
                0,
                1,
                i as i64,
                "weight",
                &Value::Double(2.0 + (i as f64)),
                100,
            )
            .unwrap();
        assert!(updated);
    }

    for i in 0..3 {
        let edge = table.get_edge(0, 1, i as i64, 100).unwrap();
        assert_eq!(edge.rank, i as i64);
    }
}

// ==================== Reverse Index Consistency ====================

#[test]
fn test_reverse_index_consistency_insert() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 0u32;
    let dst = 1u32;
    let rank = 10i64;
    let ts = 100u32;

    table
        .insert_edge(src, dst, rank, &[("weight".to_string(), Value::Double(2.5))], ts)
        .unwrap();

    let out = table.out_edges(src, ts);
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].src_vid, VertexId::from_int64(src as i64));
    assert_eq!(out[0].dst_vid, VertexId::from_int64(dst as i64));
    assert_eq!(out[0].rank, rank);

    let in_edges = table.in_edges(dst, ts);
    assert_eq!(in_edges.len(), 1);
    assert_eq!(in_edges[0].src_vid, VertexId::from_int64(src as i64));
    assert_eq!(in_edges[0].dst_vid, VertexId::from_int64(dst as i64));
    assert_eq!(in_edges[0].rank, rank);
}

#[test]
fn test_reverse_index_consistency_delete() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 0u32;
    let dst = 1u32;
    let rank = 10i64;

    table
        .insert_edge(src, dst, rank, &[("weight".to_string(), Value::Double(2.5))], 100)
        .unwrap();

    let deleted = table.delete_edge(src, dst, rank, 200).unwrap();
    assert!(deleted);

    let out = table.out_edges(src, 200);
    assert_eq!(out.len(), 0);

    let in_edges = table.in_edges(dst, 200);
    assert_eq!(in_edges.len(), 0);

    let out_old = table.out_edges(src, 100);
    assert_eq!(out_old.len(), 1);

    let in_old = table.in_edges(dst, 100);
    assert_eq!(in_old.len(), 1);
}

#[test]
fn test_reverse_index_consistency_parallel_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 0u32;
    let dst = 1u32;

    for rank in 0..3 {
        table
            .insert_edge(src, dst, rank, &[("weight".to_string(), Value::Double(rank as f64))], 100)
            .unwrap();
    }

    let out = table.out_edges(src, 100);
    assert_eq!(out.len(), 3);

    let in_edges = table.in_edges(dst, 100);
    assert_eq!(in_edges.len(), 3);

    let deleted = table.delete_edge(src, dst, 1, 200).unwrap();
    assert!(deleted);

    let out_after = table.out_edges(src, 200);
    assert_eq!(out_after.len(), 2);

    let in_after = table.in_edges(dst, 200);
    assert_eq!(in_after.len(), 2);
}

// ==================== P0 Priority Tests ====================

#[test]
fn test_p0_segment_reverse_index_sync_on_delete() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 5u32;
    let dst = 10u32;
    let rank = 100i64;

    table
        .insert_edge(src, dst, rank, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    assert!(table.has_edge(src, dst, rank, 100));
    let out_before = table.out_edges(src, 100);
    let in_before = table.in_edges(dst, 100);
    assert_eq!(out_before.len(), 1);
    assert_eq!(in_before.len(), 1);

    table.freeze_csr_only(150);

    let out_after_freeze = table.out_edges(src, 150);
    let in_after_freeze = table.in_edges(dst, 150);
    assert_eq!(out_after_freeze.len(), 1);
    assert_eq!(in_after_freeze.len(), 1);

    let deleted = table.delete_edge(src, dst, rank, 200).unwrap();
    assert!(deleted);

    let out_after_delete = table.out_edges(src, 200);
    let in_after_delete = table.in_edges(dst, 200);

    assert_eq!(out_after_delete.len(), 0);
    assert_eq!(in_after_delete.len(), 0);

    let out_old = table.out_edges(src, 150);
    let in_old = table.in_edges(dst, 150);
    assert_eq!(out_old.len(), 1);
    assert_eq!(in_old.len(), 1);
}

#[test]
fn test_p0_multi_edge_segment_delete_consistency() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 0u32;
    let dst = 1u32;

    for rank in 0..3 {
        table
            .insert_edge(src, dst, rank, &[("weight".to_string(), Value::Double(rank as f64))], 100)
            .unwrap();
    }

    table.freeze_csr_only(150);

    assert_eq!(table.out_edges(src, 150).len(), 3);
    assert_eq!(table.in_edges(dst, 150).len(), 3);

    table.delete_edge(src, dst, 1, 200).unwrap();

    assert_eq!(table.out_edges(src, 200).len(), 2);
    assert_eq!(table.in_edges(dst, 200).len(), 2);

    table.delete_edge(src, dst, 0, 200).unwrap();

    assert_eq!(table.out_edges(src, 200).len(), 1);
    assert_eq!(table.in_edges(dst, 200).len(), 1);
}
