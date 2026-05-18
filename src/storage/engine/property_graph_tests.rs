//! Tests for Property Graph Storage

use crate::core::{DataType, Value};
use crate::storage::edge::{EdgeStrategy, PropertyDef as EdgePropertyDef};
use crate::storage::engine::property_graph::{InsertEdgeParams, PropertyGraph, PropertyGraphUpdateEdgePropertyParams};
use crate::storage::vertex::PropertyDef;

#[test]
fn test_create_and_get_vertex() {
    let graph = PropertyGraph::new();
    let label_id = graph
        .create_vertex_type(
            "person",
            vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            "name",
        )
        .unwrap();

    let internal_id = graph
        .insert_vertex(
            label_id,
            "alice",
            &[
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
            ],
            100,
        )
        .unwrap();

    let vertex = graph.get_vertex(label_id, "alice", 100).unwrap();
    assert_eq!(vertex.internal_id, internal_id);
}

#[test]
fn test_create_edge() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();

    let edge_id = graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();

    let edge = graph
        .get_edge(knows_label, person_label, "alice", person_label, "bob", 100)
        .unwrap();
    assert_eq!(edge.edge_id, edge_id.0 as u64);
}

#[test]
fn test_delete_vertex() {
    let graph = PropertyGraph::new();
    let label_id = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    graph
        .insert_vertex(
            label_id,
            "alice",
            &[("name".to_string(), Value::String("Alice".to_string()))],
            100,
        )
        .unwrap();

    graph.delete_vertex(label_id, "alice", 100).unwrap();
    assert!(graph.get_vertex(label_id, "alice", 100).is_none());
}

#[test]
fn test_drop_vertex_type() {
    let graph = PropertyGraph::new();
    let _label_id = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    graph.drop_vertex_type("person").unwrap();
    assert!(graph.get_vertex_label_id("person").is_none());
}

#[test]
fn test_vertex_count() {
    let graph = PropertyGraph::new();
    let label_id = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    for i in 0..10 {
        graph
            .insert_vertex(
                label_id,
                &format!("person{}", i),
                &[(
                    "name".to_string(),
                    Value::String(format!("Person{}", i)),
                )],
                100,
            )
            .unwrap();
    }

    assert_eq!(graph.vertex_count(label_id, 100), 10);
}

#[test]
fn test_out_edges() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();
    graph
        .insert_vertex(
            person_label,
            "charlie",
            &[("name".to_string(), Value::String("Charlie".to_string()))],
            100,
        )
        .unwrap();

    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();
    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "charlie",
            properties: &[("weight".to_string(), Value::Double(2.0))],
            ts: 100,
        })
        .unwrap();

    let edges = graph
        .out_edges(knows_label, person_label, person_label, "alice", 100)
        .unwrap();
    assert_eq!(edges.len(), 2);
}

#[test]
fn test_in_edges() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();
    graph
        .insert_vertex(
            person_label,
            "charlie",
            &[("name".to_string(), Value::String("Charlie".to_string()))],
            100,
        )
        .unwrap();

    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();
    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "charlie",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(2.0))],
            ts: 100,
        })
        .unwrap();

    let edges = graph
        .in_edges(knows_label, person_label, person_label, "bob", 100)
        .unwrap();
    assert_eq!(edges.len(), 2);
}

#[test]
fn test_update_vertex_property() {
    let graph = PropertyGraph::new();
    let label_id = graph
        .create_vertex_type(
            "person",
            vec![
                PropertyDef::new("name".to_string(), DataType::String),
                PropertyDef::new("age".to_string(), DataType::Int).nullable(true),
            ],
            "name",
        )
        .unwrap();

    graph
        .insert_vertex(
            label_id,
            "alice",
            &[
                ("name".to_string(), Value::String("Alice".to_string())),
                ("age".to_string(), Value::Int(30)),
            ],
            100,
        )
        .unwrap();

    graph
        .update_vertex_property(label_id, "alice", "age", &Value::Int(31), 100)
        .unwrap();

    let vertex = graph.get_vertex(label_id, "alice", 100).unwrap();
    let age_prop = vertex.properties.iter().find(|(name, _)| name == "age").unwrap();
    assert_eq!(age_prop.1, Value::Int(31));
}

#[test]
fn test_update_edge_property() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();

    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();

    graph
        .update_edge_property(PropertyGraphUpdateEdgePropertyParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            prop_name: "weight",
            value: &Value::Double(2.0),
            ts: 100,
        })
        .unwrap();

    let edge = graph
        .get_edge(knows_label, person_label, "alice", person_label, "bob", 100)
        .unwrap();
    let weight_prop = edge.properties.iter().find(|(name, _)| name == "weight").unwrap();
    assert_eq!(weight_prop.1, Value::Double(2.0));
}

#[test]
fn test_delete_edge() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();

    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();

    let deleted = graph
        .delete_edge(
            knows_label,
            person_label,
            "alice",
            person_label,
            "bob",
            100,
        )
        .unwrap();
    assert!(deleted);

    let edge = graph.get_edge(
        knows_label,
        person_label,
        "alice",
        person_label,
        "bob",
        100,
    );
    assert!(edge.is_none());
}

#[test]
fn test_edge_count() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();
    graph
        .insert_vertex(
            person_label,
            "charlie",
            &[("name".to_string(), Value::String("Charlie".to_string()))],
            100,
        )
        .unwrap();

    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "alice",
            dst_label: person_label,
            dst_id: "bob",
            properties: &[("weight".to_string(), Value::Double(1.0))],
            ts: 100,
        })
        .unwrap();
    graph
        .insert_edge(InsertEdgeParams {
            edge_label: knows_label,
            src_label: person_label,
            src_id: "bob",
            dst_label: person_label,
            dst_id: "charlie",
            properties: &[("weight".to_string(), Value::Double(2.0))],
            ts: 100,
        })
        .unwrap();

    assert_eq!(graph.edge_count(knows_label), 2);
}

#[test]
fn test_drop_edge_type() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .unwrap();

    graph.drop_edge_type("knows").unwrap();
    assert!(graph.get_edge_label_id("knows").is_none());
}

#[test]
fn test_vertex_label_names() {
    let graph = PropertyGraph::new();
    graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();
    graph
        .create_vertex_type(
            "company",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let names = graph.vertex_label_names();
    assert_eq!(names.len(), 2);
    assert!(names.iter().any(|n| n == "person"));
    assert!(names.iter().any(|n| n == "company"));
}

#[test]
fn test_edge_label_names() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .unwrap();

    let names = graph.edge_label_names();
    assert_eq!(names.len(), 1);
    assert!(names.iter().any(|n| n == "knows"));
}

#[test]
fn test_duplicate_vertex_type() {
    let graph = PropertyGraph::new();
    graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let result = graph.create_vertex_type(
        "person",
        vec![PropertyDef::new("name".to_string(), DataType::String)],
        "name",
    );
    assert!(result.is_err());
}

#[test]
fn test_duplicate_edge_type() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
        )
        .unwrap();

    let result = graph.create_edge_type(
        "knows",
        person_label,
        person_label,
        vec![EdgePropertyDef::new(
            "weight".to_string(),
            DataType::Double,
        )],
        EdgeStrategy::Multiple,
        EdgeStrategy::Multiple,
    );
    assert!(result.is_err());
}

#[test]
fn test_edge_with_missing_vertex_label() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let result = graph.create_edge_type(
        "knows",
        person_label,
        999,
        vec![EdgePropertyDef::new(
            "weight".to_string(),
            DataType::Double,
        )],
        EdgeStrategy::Multiple,
        EdgeStrategy::Multiple,
    );
    assert!(result.is_err());
}

#[test]
fn test_insert_vertex_with_missing_label() {
    let graph = PropertyGraph::new();
    let result = graph.insert_vertex(
        999,
        "alice",
        &[("name".to_string(), Value::String("Alice".to_string()))],
        100,
    );
    assert!(result.is_err());
}

#[test]
fn test_get_nonexistent_vertex() {
    let graph = PropertyGraph::new();
    let label_id = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let result = graph.get_vertex(label_id, "nonexistent", 100);
    assert!(result.is_none());
}

#[test]
fn test_get_nonexistent_edge() {
    let graph = PropertyGraph::new();
    let person_label = graph
        .create_vertex_type(
            "person",
            vec![PropertyDef::new("name".to_string(), DataType::String)],
            "name",
        )
        .unwrap();

    let knows_label = graph
        .create_edge_type(
            "knows",
            person_label,
            person_label,
            vec![EdgePropertyDef::new(
                "weight".to_string(),
                DataType::Double,
            )],
            EdgeStrategy::Multiple,
            EdgeStrategy::Multiple,
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
            person_label,
            "bob",
            &[("name".to_string(), Value::String("Bob".to_string()))],
            100,
        )
        .unwrap();

    let result = graph.get_edge(
        knows_label,
        person_label,
        "alice",
        person_label,
        "bob",
        100,
    );
    assert!(result.is_none());
}
