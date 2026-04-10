//! 全文索引自动同步集成测试

use graphdb::coordinator::fulltext::{ChangeType, FulltextCoordinator};
use graphdb::coordinator::fulltext_sync::{register_fulltext_sync, FulltextSyncHandler};
use graphdb::core::types::Tag;
use graphdb::core::{Value, Vertex};
use graphdb::event::{EventHub, MemoryEventHub, StorageEvent};
use graphdb::search::manager::FulltextIndexManager;
use std::sync::Arc;

#[test]
fn test_event_hub_publish_subscribe() {
    let event_hub = Arc::new(MemoryEventHub::new());
    let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c = counter.clone();

    event_hub
        .subscribe(graphdb::event::EventType::VertexEvent, move |_| {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(())
        })
        .expect("subscribe should succeed");

    let event = StorageEvent::VertexInserted {
        space_id: 1,
        vertex: create_test_vertex(),
        timestamp: 0,
    };

    event_hub.publish(event).expect("publish should succeed");

    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
}

#[test]
fn test_fulltext_sync_handler_insert() {
    let manager = Arc::new(FulltextIndexManager::new());
    let coordinator = Arc::new(FulltextCoordinator::new(manager));
    let handler = FulltextSyncHandler::new(coordinator);

    let vertex = create_test_vertex_with_text();
    let event = StorageEvent::VertexInserted {
        space_id: 1,
        vertex,
        timestamp: 0,
    };

    let result = handler.handle_event(&event);
    assert!(result.is_ok(), "handle_event should succeed");
}

#[test]
fn test_fulltext_sync_handler_update() {
    let manager = Arc::new(FulltextIndexManager::new());
    let coordinator = Arc::new(FulltextCoordinator::new(manager));
    let handler = FulltextSyncHandler::new(coordinator);

    let old_vertex = create_test_vertex_with_text();
    let new_vertex = create_test_vertex_with_text_updated();
    let changed_fields = vec!["name".to_string()];

    let event = StorageEvent::VertexUpdated {
        space_id: 1,
        old_vertex,
        new_vertex,
        changed_fields,
        timestamp: 0,
    };

    let result = handler.handle_event(&event);
    assert!(result.is_ok(), "handle_event should succeed");
}

#[test]
fn test_fulltext_sync_handler_delete() {
    let manager = Arc::new(FulltextIndexManager::new());
    let coordinator = Arc::new(FulltextCoordinator::new(manager));
    let handler = FulltextSyncHandler::new(coordinator);

    let event = StorageEvent::VertexDeleted {
        space_id: 1,
        vertex_id: Value::Int64(1),
        tag_name: "Person".to_string(),
        timestamp: 0,
    };

    let result = handler.handle_event(&event);
    assert!(result.is_ok(), "handle_event should succeed");
}

#[test]
fn test_register_fulltext_sync() {
    let event_hub = Arc::new(MemoryEventHub::new());
    let manager = Arc::new(FulltextIndexManager::new());
    let coordinator = Arc::new(FulltextCoordinator::new(manager));

    let result = register_fulltext_sync(coordinator, event_hub.clone());
    assert!(result.is_ok(), "register_fulltext_sync should succeed");

    let subscription_id = result.unwrap();
    assert!(subscription_id.0 > 0, "subscription id should be valid");
}

#[test]
fn test_vertex_insert_sync_to_fulltext() {
    let event_hub = Arc::new(MemoryEventHub::new());
    let manager = Arc::new(FulltextIndexManager::new());
    let coordinator = Arc::new(FulltextCoordinator::new(manager.clone()));

    register_fulltext_sync(coordinator, event_hub.clone()).expect("register should succeed");

    let vertex = create_test_vertex_with_text();
    let event = StorageEvent::VertexInserted {
        space_id: 1,
        vertex: vertex.clone(),
        timestamp: 0,
    };

    event_hub.publish(event).expect("publish should succeed");

    let doc_id = vertex.vid.to_string();
    let tag = &vertex.tags[0];

    for (field_name, value) in &tag.properties {
        if let Value::String(_) = value {
            let engine = manager.get_engine(1, &tag.name, field_name);
            assert!(
                engine.is_some(),
                "engine should exist for field {}",
                field_name
            );
        }
    }
}

fn create_test_vertex() -> Vertex {
    Vertex {
        vid: Value::Int64(1),
        tags: vec![],
    }
}

fn create_test_vertex_with_text() -> Vertex {
    let mut properties = std::collections::HashMap::new();
    properties.insert("name".to_string(), Value::String("Alice".to_string()));
    properties.insert(
        "description".to_string(),
        Value::String("A test person".to_string()),
    );

    let tag = Tag {
        name: "Person".to_string(),
        properties,
    };

    Vertex {
        vid: Value::Int64(1),
        tags: vec![tag],
    }
}

fn create_test_vertex_with_text_updated() -> Vertex {
    let mut properties = std::collections::HashMap::new();
    properties.insert(
        "name".to_string(),
        Value::String("Alice Updated".to_string()),
    );
    properties.insert(
        "description".to_string(),
        Value::String("Updated description".to_string()),
    );

    let tag = Tag {
        name: "Person".to_string(),
        properties,
    };

    Vertex {
        vid: Value::Int64(1),
        tags: vec![tag],
    }
}
