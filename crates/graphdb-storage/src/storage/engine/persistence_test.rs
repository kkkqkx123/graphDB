#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::core::{DataType, Value};
    use crate::storage::edge::EdgeStrategy;
    use crate::storage::engine::config::PropertyGraphConfig;
    use crate::storage::engine::graph_storage::GraphStorageContext;
    use crate::storage::engine::params::{
        CreateEdgeTypeParams, EdgeOperationParams, InsertEdgeParams,
    };
    use crate::storage::types::StoragePropertyDef;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir()
            .join("graphdb_persistence_test")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_flush_tables_to_dir_custom_path() {
        let dir = temp_dir("flush_custom");
        let data_dir = dir.join("custom_data");

        let config = PropertyGraphConfig::default()
            .with_work_dir(dir.clone())
            .with_cache(true, 1024 * 1024);
        let graph = GraphStorageContext::with_config(config);

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
}
