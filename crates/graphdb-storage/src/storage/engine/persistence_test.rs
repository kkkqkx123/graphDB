#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::{DataType, Value};
    use crate::storage::edge::EdgeStrategy;
    use crate::storage::engine::config::PropertyGraphConfig;
    use crate::storage::engine::edge_params::CreateEdgeTypeParams;
    use crate::storage::engine::property_graph::{InsertEdgeParams, PropertyGraph};
    use crate::storage::storage_types::StoragePropertyDef;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("graphdb_persistence_test")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_flush_and_load_round_trip() {
        let dir = temp_dir("round_trip");

        // Phase 1: Create graph, insert data, flush
        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = PropertyGraph::with_config(config);

        let person_label = graph
            .create_vertex_type(
                "person",
                vec![
                    StoragePropertyDef::new("name".to_string(), DataType::String),
                    StoragePropertyDef::new("age".to_string(), DataType::Int).nullable(true),
                ],
                "name",
            )
            .unwrap();

        graph
            .insert_vertex(
                person_label,
                "alice",
                &[
                    ("name".to_string(), Value::String("Alice".to_string())),
                    ("age".to_string(), Value::Int(30)),
                ],
                100,
            )
            .unwrap();
        graph
            .insert_vertex(
                person_label,
                "bob",
                &[
                    ("name".to_string(), Value::String("Bob".to_string())),
                    ("age".to_string(), Value::Int(25)),
                ],
                100,
            )
            .unwrap();

        let knows_label = graph
            .create_edge_type(
                "knows",
                person_label,
                person_label,
                vec![StoragePropertyDef::new(
                    "weight".to_string(),
                    DataType::Double,
                )],
                EdgeStrategy::Multiple,
                EdgeStrategy::Multiple,
            )
            .unwrap();

        graph
            .insert_edge(InsertEdgeParams {
                edge_label: knows_label,
                src_label: person_label,
                src_id: "alice",
                dst_label: person_label,
                dst_id: "bob",
                properties: &[("weight".to_string(), Value::Double(0.95))],
                ts: 100,
            })
            .unwrap();

        // Flush to disk
        graph.flush_to_disk().unwrap();

        assert_eq!(graph.vertex_count(person_label, 100), 2);

        // Phase 2: Create fresh graph in same dir, load data
        let config2 = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph2 = PropertyGraph::with_config(config2);

        graph2
            .create_vertex_type_with_id(
                "person",
                person_label,
                vec![
                    StoragePropertyDef::new("name".to_string(), DataType::String),
                    StoragePropertyDef::new("age".to_string(), DataType::Int).nullable(true),
                ],
                "name",
            )
            .unwrap();
        graph2
            .create_edge_type_with_id(
                CreateEdgeTypeParams {
                    name: "knows",
                    src_label: person_label,
                    dst_label: person_label,
                    properties: vec![StoragePropertyDef::new(
                        "weight".to_string(),
                        DataType::Double,
                    )],
                    oe_strategy: EdgeStrategy::Multiple,
                    ie_strategy: EdgeStrategy::Multiple,
                },
                knows_label,
            )
            .unwrap();

        // Load data from disk
        graph2.load_data().unwrap();

        // Verify data survived round-trip
        let alice = graph2
            .get_vertex(person_label, "alice", 100)
            .expect("Alice should exist after reload");
        assert_eq!(
            alice
                .properties
                .iter()
                .find(|(n, _)| n == "name")
                .map(|(_, v)| v.clone()),
            Some(Value::String("Alice".to_string()))
        );

        let edge = graph2
            .get_edge(knows_label, person_label, "alice", person_label, "bob", 100)
            .expect("Edge should exist after reload");
        assert_eq!(
            edge.properties
                .iter()
                .find(|(n, _)| n == "weight")
                .map(|(_, v)| v.clone()),
            Some(Value::Double(0.95))
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_flush_incremental_tracks_modifications() {
        let dir = temp_dir("flush_incremental");
        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = PropertyGraph::with_config(config);

        let person_label = graph
            .create_vertex_type(
                "person",
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();

        graph
            .insert_vertex(
                person_label,
                "alice",
                &[("name".to_string(), Value::String("Alice".to_string()))],
                100,
            )
            .unwrap();

        // Full flush first
        graph.flush_to_disk().unwrap();

        // Insert one more vertex
        graph
            .insert_vertex(
                person_label,
                "bob",
                &[("name".to_string(), Value::String("Bob".to_string()))],
                101,
            )
            .unwrap();

        // Incremental flush
        let modified = graph.flush_incremental().unwrap();
        assert!(!modified.is_empty(), "Should have tracked modified tables");

        // Verify incremental flush data
        let config2 = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph2 = PropertyGraph::with_config(config2);
        graph2
            .create_vertex_type_with_id(
                "person",
                person_label,
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();
        graph2.load_data().unwrap();

        let bob = graph2.get_vertex(person_label, "bob", 101);
        assert!(
            bob.is_some(),
            "Bob should exist after incremental flush reload"
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_flush_tables_to_dir_custom_path() {
        let dir = temp_dir("flush_custom");
        let data_dir = dir.join("custom_data");

        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = PropertyGraph::with_config(config);

        let person_label = graph
            .create_vertex_type(
                "person",
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();

        graph
            .insert_vertex(
                person_label,
                "alice",
                &[("name".to_string(), Value::String("Alice".to_string()))],
                100,
            )
            .unwrap();

        // Flush to custom dir
        graph.flush_tables_to_dir(&data_dir).unwrap();

        assert!(data_dir.join("vertices").exists());
        assert!(data_dir.join("edges").exists());

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_flush_and_load_multiple_vertex_types() {
        let dir = temp_dir("multi_type");
        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = PropertyGraph::with_config(config);

        let person_label = graph
            .create_vertex_type(
                "person",
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();
        let company_label = graph
            .create_vertex_type(
                "company",
                vec![
                    StoragePropertyDef::new("name".to_string(), DataType::String),
                    StoragePropertyDef::new("revenue".to_string(), DataType::Double).nullable(true),
                ],
                "name",
            )
            .unwrap();

        graph
            .insert_vertex(
                person_label,
                "alice",
                &[("name".to_string(), Value::String("Alice".to_string()))],
                100,
            )
            .unwrap();
        graph
            .insert_vertex(
                company_label,
                "acme",
                &[
                    ("name".to_string(), Value::String("Acme Inc".to_string())),
                    ("revenue".to_string(), Value::Double(1_000_000.0)),
                ],
                100,
            )
            .unwrap();

        graph.flush_to_disk().unwrap();

        // Reload
        let config2 = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph2 = PropertyGraph::with_config(config2);
        graph2
            .create_vertex_type_with_id(
                "person",
                person_label,
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();
        graph2
            .create_vertex_type_with_id(
                "company",
                company_label,
                vec![
                    StoragePropertyDef::new("name".to_string(), DataType::String),
                    StoragePropertyDef::new("revenue".to_string(), DataType::Double).nullable(true),
                ],
                "name",
            )
            .unwrap();
        graph2.load_data().unwrap();

        let alice = graph2.get_vertex(person_label, "alice", 100);
        assert!(alice.is_some());
        let acme = graph2.get_vertex(company_label, "acme", 100);
        assert!(acme.is_some());
        assert_eq!(
            acme.as_ref()
                .unwrap()
                .properties
                .iter()
                .find(|(n, _)| n == "revenue")
                .map(|(_, v)| v.clone()),
            Some(Value::Double(1_000_000.0))
        );

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_flush_preserves_data_across_reloads() {
        let dir = temp_dir("preserve_reload");
        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = PropertyGraph::with_config(config);

        let person_label = graph
            .create_vertex_type(
                "person",
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();

        // Insert multiple vertices and flush
        for i in 0..5 {
            graph
                .insert_vertex(
                    person_label,
                    &format!("user_{}", i),
                    &[("name".to_string(), Value::String(format!("User_{}", i)))],
                    (100 + i) as u32,
                )
                .unwrap();
        }
        graph.flush_to_disk().unwrap();

        // Reload and check
        let config2 = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph2 = PropertyGraph::with_config(config2);
        graph2
            .create_vertex_type_with_id(
                "person",
                person_label,
                vec![StoragePropertyDef::new(
                    "name".to_string(),
                    DataType::String,
                )],
                "name",
            )
            .unwrap();
        graph2.load_data().unwrap();

        for i in 0..5 {
            let v = graph2.get_vertex(person_label, &format!("user_{}", i), 100);
            assert!(v.is_some(), "user_{} should exist after reload", i);
        }

        // Cleanup
        let _ = std::fs::remove_dir_all(&dir);
    }
}
