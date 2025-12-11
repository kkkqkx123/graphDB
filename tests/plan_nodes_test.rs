//! 测试计划节点的实现

use graphdb::query::planner::plan::*;

#[test]
fn test_start_node_creation() {
    let start_node = StartNode::new(1);
    assert_eq!(start_node.id(), 1);
    assert_eq!(start_node.kind(), PlanNodeKind::Start);
    assert_eq!(start_node.dependencies().len(), 0);
}

#[test]
fn test_multi_shortest_path_node_creation() {
    let start_node = StartNode::new(1);
    let another_start = StartNode::new(2);

    let multi_shortest =
        MultiShortestPath::new(3, Box::new(start_node), Box::new(another_start), 5);

    assert_eq!(multi_shortest.id(), 3);
    assert_eq!(multi_shortest.kind(), PlanNodeKind::MultiShortestPath);
    assert_eq!(multi_shortest.steps(), 5);
    assert_eq!(multi_shortest.dependencies().len(), 2);
}

#[test]
fn test_create_space_node_creation() {
    let create_space = CreateSpace::new(1, true, "test_space", 10, 3);

    assert_eq!(create_space.id(), 1);
    assert_eq!(create_space.kind(), PlanNodeKind::CreateSpace);
    assert_eq!(create_space.if_not_exist, true);
    assert_eq!(create_space.space_name, "test_space");
    assert_eq!(create_space.partition_num, 10);
    assert_eq!(create_space.replica_factor, 3);
}

#[test]
fn test_insert_vertices_node_creation() {
    let insert_vertices = InsertVertices::new(
        1,
        1001, // space_id
        101, // tag_id
        Vec::new(), // props
        true, // insertable
    );

    assert_eq!(insert_vertices.id(), 1);
    assert_eq!(insert_vertices.kind(), PlanNodeKind::InsertVertices);
    assert_eq!(insert_vertices.space_id, 1001);
    assert_eq!(insert_vertices.insertable, true);
}

#[test]
fn test_select_node_creation() {
    let start_node = StartNode::new(1);
    let mut select_node = SelectNode::new(2, "age > 18");

    select_node.set_if_branch(Box::new(start_node));

    assert_eq!(select_node.id(), 2);
    assert_eq!(select_node.kind(), PlanNodeKind::Select);
    assert_eq!(select_node.condition, "age > 18");
    assert!(select_node.if_branch().is_some());
}

#[test]
fn test_submit_job_node_creation() {
    let submit_job = SubmitJob::new(
        1,
        JobType::Compaction,
        vec!["param1".to_string(), "param2".to_string()],
    );

    assert_eq!(submit_job.id(), 1);
    assert_eq!(submit_job.kind(), PlanNodeKind::SubmitJob);
    assert!(matches!(submit_job.job_type(), &JobType::Compaction));
    assert_eq!(submit_job.parameters().len(), 2);
}

#[test]
fn test_update_vertex_node_creation() {
    let update_vertex = UpdateVertex::new(
        1,
        1001, // space_id
        101, // tag_id
        None, // filter
        Vec::new(), // update_props
        true, // insertable
    );

    assert_eq!(update_vertex.id(), 1);
    assert_eq!(update_vertex.kind(), PlanNodeKind::UpdateVertex);
    assert_eq!(update_vertex.space_id, 1001);
    assert_eq!(update_vertex.tag_id, 101);
}
