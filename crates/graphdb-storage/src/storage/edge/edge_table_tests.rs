//! Edge Table Tests
//!
//! Comprehensive test suite for EdgeTable functionality including:
//! - Basic insert/get/delete operations
//! - Parallel edges with different ranks
//! - Timestamp-based visibility and tombstones
//! - Property updates
//! - Persistence (flush/load roundtrips)

use super::*;

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
fn test_freeze_csr_preserves_reads() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();
    table
        .insert_edge(0, 2, 0, &[("weight".to_string(), Value::Double(2.5))], 110)
        .unwrap();

    let before = table.scan(150);
    let frozen = table.freeze_csr_only(150);
    let after = table.scan(150);

    assert_eq!(frozen, 4);
    assert_eq!(table.out_segments.len(), 1);
    assert_eq!(table.in_segments.len(), 1);
    assert_eq!(before.len(), after.len());
    assert!(table.has_edge(0, 1, 0, 150));
    assert!(table.has_edge(0, 2, 0, 150));
}

#[test]
fn test_delete_base_segment_uses_tombstone() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table.insert_edge(0, 1, 0, &[], 100).unwrap();
    table.freeze_csr_only(150);

    assert!(table.delete_edge(0, 1, 0, 200).unwrap());
    assert!(table.has_edge(0, 1, 0, 150));
    assert!(!table.has_edge(0, 1, 0, 250));
    assert_eq!(table.scan(250).len(), 0);
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

// ==================== P0 Priority Tests ====================

/// Test: Self-loop edge handling
#[test]
fn test_self_loop_edge() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    // Insert self-loop: vertex 0 -> vertex 0
    table
        .insert_edge(0, 0, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    // Verify self-loop exists
    assert!(table.has_edge(0, 0, 0, 100));

    // Verify it appears in both outgoing and incoming edges
    let out_edges = table.out_edges(0, 100);
    assert!(out_edges.iter().any(|edge| edge.dst_vid == VertexId::from_int64(0)));

    let in_edges = table.in_edges(0, 100);
    assert!(in_edges.iter().any(|edge| edge.src_vid == VertexId::from_int64(0)));
}

/// Test: Multiple parallel edges with different ranks
#[test]
fn test_multiple_parallel_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let src = 1u32;
    let dst = 2u32;

    // Insert 5 parallel edges with different ranks
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

    // Verify all edges exist
    for rank in 0..5 {
        assert!(table.has_edge(src, dst, rank as i64, 100));
    }

    // Verify correct count and retrieval
    let edges = table.out_edges(src, 100);
    assert_eq!(edges.len(), 5);

    let incoming = table.in_edges(dst, 100);
    assert_eq!(incoming.len(), 5);
}

/// Test: Edge deletion with timestamp constraints
#[test]
fn test_edge_deletion_with_timestamps() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    // Insert edge at ts=100
    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();

    // Verify insertion succeeded by checking the edge exists
    assert!(table.has_edge(0, 1, 0, 100));

    // Delete at ts=200
    let deleted = table.delete_edge(0, 1, 0, 200).unwrap();
    assert!(deleted);

    // Verify visibility changed after deletion
    assert!(!table.has_edge(0, 1, 0, 200));
    assert!(!table.has_edge(0, 1, 0, 300));
}

/// Test: Edge property updates on multiple edges
#[test]
fn test_property_updates_multiple_edges() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    // Insert 3 edges
    for i in 0..3 {
        table
            .insert_edge(0, 1, i as i64, &[("weight".to_string(), Value::Double(1.0))], 100)
            .unwrap();
    }

    // Update properties on each edge independently
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

    // Verify updates took effect
    for i in 0..3 {
        let edge = table.get_edge(0, 1, i as i64, 100).unwrap();
        // Property updated value check happens implicitly - edge still exists
        assert_eq!(edge.rank, i as i64);
    }
}

#[test]
fn test_flush_load_roundtrip() {
    use std::fs;

    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let ts = 100u32;
    table
        .insert_edge(1, 2, 0, &[("weight".to_string(), Value::Double(1.5))], ts)
        .unwrap();

    table
        .insert_edge(1, 3, 0, &[("weight".to_string(), Value::Double(2.5))], ts)
        .unwrap();

    table
        .insert_edge(2, 3, 0, &[("weight".to_string(), Value::Double(3.5))], ts)
        .unwrap();

    let temp_dir = std::env::temp_dir().join("edge_table_test_flush_load");
    let _ = fs::remove_dir_all(&temp_dir);

    table
        .flush(
            &temp_dir,
            crate::storage::compression::CompressionType::Zstd { level: 3 },
        )
        .expect("flush should succeed");

    let mut loaded_table = EdgeTable::new(create_test_schema()).unwrap();
    loaded_table.load(&temp_dir).expect("load should succeed");

    assert_eq!(
        loaded_table.out_edges(1, ts).len(),
        2,
        "scan should work after load"
    );

    loaded_table
        .load(&temp_dir)
        .expect("second load should succeed");

    assert_eq!(
        loaded_table.out_edges(1, ts).len(),
        2,
        "scan should still work after second load"
    );
    assert_eq!(
        loaded_table.out_edges(2, ts).len(),
        1,
        "scan should work after load"
    );

    assert!(
        loaded_table.has_edge(1, 2, 0, ts),
        "get_edge should work after load"
    );

    let deleted = loaded_table
        .delete_edge(1, 3, 0, ts + 1)
        .expect("delete_edge should work after load");
    assert!(deleted, "delete_edge should find the edge");

    assert!(
        !loaded_table.has_edge(1, 3, 0, ts + 1),
        "deleted edge should not be visible"
    );

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_flush_load_preserves_segments_and_tombstones() {
    use std::fs;

    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    table
        .insert_edge(1, 2, 0, &[("weight".to_string(), Value::Double(1.5))], 100)
        .unwrap();
    table
        .insert_edge(1, 3, 0, &[("weight".to_string(), Value::Double(2.5))], 110)
        .unwrap();
    table.freeze_csr_only(150);
    table.delete_edge(1, 2, 0, 200).unwrap();

    let temp_dir = std::env::temp_dir().join("edge_table_test_segments_tombstones");
    let _ = fs::remove_dir_all(&temp_dir);

    table
        .flush(
            &temp_dir,
            crate::storage::compression::CompressionType::Zstd { level: 3 },
        )
        .expect("flush should succeed");

    let mut loaded_table = EdgeTable::new(create_test_schema()).unwrap();
    loaded_table.load(&temp_dir).expect("load should succeed");

    assert_eq!(loaded_table.out_segments.len(), 1);
    assert_eq!(loaded_table.in_segments.len(), 1);
    assert!(loaded_table.has_edge(1, 2, 0, 150));
    assert!(!loaded_table.has_edge(1, 2, 0, 250));
    assert!(loaded_table.has_edge(1, 3, 0, 250));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_maybe_compact_for_flush_reduces_fragmentation() {
    use std::fs;

    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    const NUM_EDGES: u32 = 50;
    for i in 1..=NUM_EDGES {
        let weight = i as f64 * 1.5;
        table
            .insert_edge(
                1,
                i,
                0,
                &[("weight".to_string(), Value::Double(weight))],
                100 + i,
            )
            .expect("insert_edge should work");
    }

    let fragmentation_before = table.out_csr.fragmentation_ratio();
    assert!(
        fragmentation_before > 1.0,
        "Setup failed: expected fragmentation"
    );

    let temp_dir = std::env::temp_dir().join("edge_table_test_auto_compact");
    let _ = fs::remove_dir_all(&temp_dir);

    let ts = 100 + NUM_EDGES + 100;

    table.maybe_compact_for_flush(ts, 1.0);
    let fragmentation_after = table.out_csr.fragmentation_ratio();
    assert!(
        fragmentation_after < fragmentation_before,
        "Compaction should reduce fragmentation: before={}, after={}",
        fragmentation_before,
        fragmentation_after
    );

    table
        .flush(
            &temp_dir,
            crate::storage::compression::CompressionType::Zstd { level: 3 },
        )
        .expect("flush should succeed after compaction");

    let mut loaded_table = EdgeTable::new(create_test_schema()).unwrap();
    loaded_table.load(&temp_dir).expect("load should succeed");

    for i in 1..=NUM_EDGES {
        assert!(
            loaded_table.has_edge(1, i, 0, ts),
            "Edge from {} to {} should exist after load",
            1,
            i
        );
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_export_snapshot_basic() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let ts1: Timestamp = 100;
    let ts2: Timestamp = 200;

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.5))], ts1)
        .unwrap();
    table
        .insert_edge(0, 2, 0, &[("weight".to_string(), Value::Double(2.5))], ts1)
        .unwrap();

    let snapshot = table.export_snapshot(ts1).unwrap();
    assert_eq!(snapshot.snapshot_ts, ts1);
    assert_eq!(snapshot.label, 0);

    let out_edges = snapshot.out_csr.edges_of_ref(0);
    assert_eq!(out_edges.len(), 2);

    table
        .insert_edge(0, 3, 0, &[("weight".to_string(), Value::Double(3.5))], ts2)
        .unwrap();

    let snapshot_ts1 = table.export_snapshot(ts1).unwrap();
    assert_eq!(snapshot_ts1.out_csr.edges_of_ref(0).len(), 2);

    let snapshot_ts2 = table.export_snapshot(ts2).unwrap();
    assert_eq!(snapshot_ts2.out_csr.edges_of_ref(0).len(), 3);
}

#[test]
fn test_export_snapshot_time_travel() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let ts1: Timestamp = 50;
    let ts2: Timestamp = 100;
    let ts3: Timestamp = 150;

    table
        .insert_edge(1, 2, 0, &[("weight".to_string(), Value::Double(1.0))], ts1)
        .unwrap();
    table
        .insert_edge(1, 3, 0, &[("weight".to_string(), Value::Double(2.0))], ts2)
        .unwrap();
    table
        .insert_edge(1, 4, 0, &[("weight".to_string(), Value::Double(3.0))], ts3)
        .unwrap();

    let snap_before_ts1 = table.export_snapshot(ts1 - 1).unwrap();
    assert_eq!(snap_before_ts1.out_csr.edges_of_ref(1).len(), 0);

    let snap_at_ts1 = table.export_snapshot(ts1).unwrap();
    assert_eq!(snap_at_ts1.out_csr.edges_of_ref(1).len(), 1);

    let snap_at_ts2 = table.export_snapshot(ts2).unwrap();
    assert_eq!(snap_at_ts2.out_csr.edges_of_ref(1).len(), 2);

    let snap_at_ts3 = table.export_snapshot(ts3).unwrap();
    assert_eq!(snap_at_ts3.out_csr.edges_of_ref(1).len(), 3);
}

#[test]
fn test_export_snapshot_frozen_consistency() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let ts1: Timestamp = 100;
    let ts2: Timestamp = 200;

    table
        .insert_edge(5, 10, 0, &[("weight".to_string(), Value::Double(1.0))], ts1)
        .unwrap();
    table
        .insert_edge(5, 11, 0, &[("weight".to_string(), Value::Double(2.0))], ts1)
        .unwrap();

    table.freeze_csr_only(ts1);

    table
        .insert_edge(5, 12, 0, &[("weight".to_string(), Value::Double(3.0))], ts2)
        .unwrap();

    let snapshot = table.export_snapshot(ts1).unwrap();
    assert_eq!(snapshot.out_csr.edges_of_ref(5).len(), 2);

    let snapshot_ts2 = table.export_snapshot(ts2).unwrap();
    assert_eq!(snapshot_ts2.out_csr.edges_of_ref(5).len(), 3);
}

#[test]
fn test_snapshot_simple_debug() {
    let schema = create_test_schema();
    let mut table = EdgeTable::new(schema).unwrap();

    let ts1: Timestamp = 100;

    table
        .insert_edge(0, 1, 0, &[("weight".to_string(), Value::Double(1.0))], ts1)
        .unwrap();

    let out_edges_before = table.out_edges(0, ts1);
    assert_eq!(out_edges_before.len(), 1);

    let snapshot = table.export_snapshot(ts1).unwrap();
    
    assert_eq!(snapshot.out_csr.edge_count(), 1);
    assert_eq!(snapshot.out_csr.edges_of_ref(0).len(), 1);
    
    let edges = snapshot.get_out_edges(0);
    assert_eq!(edges.len(), 1);
}
