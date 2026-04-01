//! Data Modification Executor Builder
//!
//! Responsible for creating executors for data modification operations (InsertVertices, InsertEdges, Remove).

use crate::core::error::QueryError;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_modification::{InsertExecutor, RemoveExecutor, RemoveItem};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::{
    DeleteEdgesNode, DeleteVerticesNode, InsertEdgesNode, InsertVerticesNode, RemoveNode,
    UpdateEdgesNode, UpdateNode, UpdateTargetType, UpdateVerticesNode,
};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// Data Modification Executor Builder
pub struct DataModificationBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataModificationBuilder<S> {
    /// Create a new data modification builder.
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Constructing the InsertVertices executor
    pub fn build_insert_vertices(
        node: &InsertVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Convert node data into vertex data.
        let mut vertices = Vec::new();

        for (vid_expr, tag_values_list) in node.values() {
            // Obtain the vertex ID expression and evaluate it
            let vid = vid_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("顶点ID表达式不存在或不是字面量".to_string())
                })?;

            // Obtain the tag name
            let tag_names = node.tag_names();

            // Create a list of tags.
            let mut tags = Vec::new();

            // Add tags and attributes
            for (tag_idx, tag_values) in tag_values_list.iter().enumerate() {
                if let Some(tag_name) = tag_names.get(tag_idx) {
                    // Create a mapping of tag attributes.
                    let mut tag_props = HashMap::new();

                    // Add attributes
                    if let Some(prop_names) = node.prop_names() {
                        for (prop_idx, prop_value) in tag_values.iter().enumerate() {
                            if let Some(prop_name) = prop_names.get(prop_idx) {
                                // Evaluate the expression to get the actual value
                                let value = prop_value
                                    .get_expression()
                                    .and_then(|e| Self::evaluate_literal(&e))
                                    .unwrap_or(Value::Null(crate::core::NullType::Null));
                                tag_props.insert(prop_name.clone(), value);
                            }
                        }
                    }

                    // Create tags
                    let tag = Tag::new(tag_name.clone(), tag_props);
                    tags.push(tag);
                }
            }

            // Create vertices with evaluated ID
            let vertex = Vertex::new(vid, tags);
            vertices.push(vertex);
        }

        // Create executor based on if_not_exists flag
        let executor = if node.if_not_exists() {
            InsertExecutor::with_vertices_if_not_exists(
                node.id(),
                storage,
                node.space_name().to_string(),
                vertices,
                context.expression_context().clone(),
            )
        } else {
            InsertExecutor::with_vertices(
                node.id(),
                storage,
                node.space_name().to_string(),
                vertices,
                context.expression_context().clone(),
            )
        };

        Ok(ExecutorEnum::InsertVertices(executor))
    }

    /// Evaluate a literal expression to get its value
    fn evaluate_literal(expr: &crate::core::Expression) -> Option<Value> {
        match expr {
            crate::core::Expression::Literal(value) => Some(value.clone()),
            _ => None,
        }
    }

    /// Constructing the InsertEdges executor
    pub fn build_insert_edges(
        node: &InsertEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let mut edges = Vec::new();

        for (src_expr, dst_expr, rank_expr, prop_values) in node.edges() {
            // Obtain the expression for the ID of the source vertex and evaluate it.
            let src = src_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("源顶点ID表达式不存在或不是字面量".to_string())
                })?;

            // Obtain the expression for the target vertex ID and evaluate it.
            let dst = dst_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("目标顶点ID表达式不存在或不是字面量".to_string())
                })?;

            // Obtain the rank (optional); the default value is 0.
            let rank = rank_expr
                .as_ref()
                .and_then(|e| e.get_expression())
                .and_then(|expr| Self::evaluate_literal(&expr))
                .and_then(|v| match v {
                    crate::core::Value::Int(v) => Some(v),
                    _ => None,
                })
                .unwrap_or(0);

            // Create a mapping of edge attributes.
            let mut props = HashMap::new();
            let prop_names = node.prop_names();
            for (prop_idx, prop_value) in prop_values.iter().enumerate() {
                if let Some(prop_name) = prop_names.get(prop_idx) {
                    if let Some(value_expr) = prop_value.get_expression() {
                        // Evaluate the expression to get the actual value
                        let value = Self::evaluate_literal(&value_expr)
                            .unwrap_or(Value::Null(crate::core::NullType::Null));
                        props.insert(prop_name.clone(), value);
                    }
                }
            }

            // Create an edge with evaluated src, dst and rank
            let edge = Edge::new(src, dst, node.edge_name().to_string(), rank, props);

            edges.push(edge);
        }

        // Create executor based on if_not_exists flag
        let executor = if node.if_not_exists() {
            InsertExecutor::with_edges_if_not_exists(
                node.id(),
                storage,
                node.space_name().to_string(),
                edges,
                context.expression_context().clone(),
            )
        } else {
            InsertExecutor::with_edges(
                node.id(),
                storage,
                node.space_name().to_string(),
                edges,
                context.expression_context().clone(),
            )
        };

        Ok(ExecutorEnum::InsertEdges(executor))
    }

    /// Building the Remove Executor
    pub fn build_remove(
        node: &RemoveNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Translate: `remove_items`
        let remove_items: Vec<RemoveItem> = node
            .remove_items()
            .iter()
            .map(|(item_type, expr)| {
                let item_type_enum = if item_type == "property" {
                    crate::query::executor::data_modification::RemoveItemType::Property
                } else {
                    crate::query::executor::data_modification::RemoveItemType::Tag
                };
                RemoveItem {
                    item_type: item_type_enum,
                    expression: expr.clone(),
                }
            })
            .collect();

        let executor = RemoveExecutor::new(
            node.id(),
            storage,
            remove_items,
            context.expression_context().clone(),
        );

        Ok(ExecutorEnum::Remove(executor))
    }

    /// Building the DeleteVertices executor
    pub fn build_delete_vertices(
        node: &DeleteVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_modification::DeleteExecutor;

        // Convert vertex ID expressions to values
        let mut vertex_ids = Vec::new();
        for vid_expr in node.vertex_ids() {
            let vid = vid_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("顶点ID表达式不存在或不是字面量".to_string())
                })?;
            vertex_ids.push(vid);
        }

        let executor = DeleteExecutor::new(
            node.id(),
            storage,
            Some(vertex_ids),
            None, // edge_ids
            None, // condition
            context.expression_context().clone(),
        )
        .with_space(node.space_name().to_string())
        .with_edge(node.with_edge());

        Ok(ExecutorEnum::Delete(executor))
    }

    /// Building the DeleteEdges executor
    pub fn build_delete_edges(
        node: &DeleteEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_modification::DeleteExecutor;

        // Convert edge expressions to (src, dst, edge_type) tuples
        let mut edge_ids = Vec::new();
        for (src_expr, dst_expr, _rank_expr) in node.edges() {
            let src = src_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("源顶点ID表达式不存在或不是字面量".to_string())
                })?;

            let dst = dst_expr
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("目标顶点ID表达式不存在或不是字面量".to_string())
                })?;

            // Use the edge type from the node, or a default if not specified
            let edge_type = node.edge_type().unwrap_or("UNKNOWN").to_string();

            edge_ids.push((src, dst, edge_type));
        }

        let executor = DeleteExecutor::new(
            node.id(),
            storage,
            None, // vertex_ids
            Some(edge_ids),
            None, // condition
            context.expression_context().clone(),
        )
        .with_space(node.space_name().to_string());

        Ok(ExecutorEnum::Delete(executor))
    }

    /// Building the Update executor
    pub fn build_update(
        node: &UpdateNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_modification::{EdgeUpdate, UpdateExecutor, VertexUpdate};

        match node.info() {
            UpdateTargetType::Vertex(info) => {
                let vertex_id = info
                    .vertex_id
                    .get_expression()
                    .and_then(|e| Self::evaluate_literal(&e))
                    .ok_or_else(|| {
                        QueryError::ExecutionError("顶点ID表达式不存在或不是字面量".to_string())
                    })?;

                log::debug!(
                    "[build_update] vertex_id={:?}, properties_count={}",
                    vertex_id,
                    info.properties.len()
                );
                eprintln!(
                    "[build_update] vertex_id={:?}, properties_count={}",
                    vertex_id,
                    info.properties.len()
                );

                let mut properties = HashMap::new();
                for (key, value_expr) in &info.properties {
                    let expr_opt = value_expr.get_expression();
                    log::debug!(
                        "[build_update] property '{}' expression={:?}",
                        key,
                        expr_opt
                    );
                    eprintln!(
                        "[build_update] property '{}' expression={:?}",
                        key, expr_opt
                    );
                    let value = expr_opt
                        .and_then(|e| Self::evaluate_literal(&e))
                        .ok_or_else(|| {
                            QueryError::ExecutionError(format!(
                                "属性 {} 的值表达式不存在或不是字面量",
                                key
                            ))
                        })?;
                    properties.insert(key.clone(), value);
                }

                log::debug!("[build_update] final properties={:?}", properties);

                let vertex_update = VertexUpdate {
                    vertex_id,
                    properties,
                    tags_to_add: None,
                    tags_to_remove: None,
                };

                let executor = UpdateExecutor::new(
                    node.id(),
                    storage,
                    Some(vec![vertex_update]),
                    None,
                    info.condition.clone(),
                    context.expression_context().clone(),
                )
                .with_space(info.space_name.clone())
                .with_insertable(info.is_upsert);

                Ok(ExecutorEnum::Update(executor))
            }
            UpdateTargetType::Edge(info) => {
                let src = info
                    .src
                    .get_expression()
                    .and_then(|e| Self::evaluate_literal(&e))
                    .ok_or_else(|| {
                        QueryError::ExecutionError("源顶点ID表达式不存在或不是字面量".to_string())
                    })?;

                let dst = info
                    .dst
                    .get_expression()
                    .and_then(|e| Self::evaluate_literal(&e))
                    .ok_or_else(|| {
                        QueryError::ExecutionError("目标顶点ID表达式不存在或不是字面量".to_string())
                    })?;

                let rank = info
                    .rank
                    .as_ref()
                    .and_then(|r| r.get_expression().and_then(|e| Self::evaluate_literal(&e)))
                    .and_then(|v| match v {
                        Value::Int(i) => Some(i),
                        _ => None,
                    });

                let mut properties = HashMap::new();
                for (key, value_expr) in &info.properties {
                    let value = value_expr
                        .get_expression()
                        .and_then(|e| Self::evaluate_literal(&e))
                        .ok_or_else(|| {
                            QueryError::ExecutionError(format!(
                                "属性 {} 的值表达式不存在或不是字面量",
                                key
                            ))
                        })?;
                    properties.insert(key.clone(), value);
                }

                let edge_type = info.edge_type.clone().unwrap_or_default();

                let edge_update = EdgeUpdate {
                    src,
                    dst,
                    edge_type,
                    rank,
                    properties,
                };

                let executor = UpdateExecutor::new(
                    node.id(),
                    storage,
                    None,
                    Some(vec![edge_update]),
                    info.condition.clone(),
                    context.expression_context().clone(),
                )
                .with_space(info.space_name.clone())
                .with_insertable(info.is_upsert);

                Ok(ExecutorEnum::Update(executor))
            }
        }
    }

    /// Building the UpdateVertices executor
    pub fn build_update_vertices(
        node: &UpdateVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_modification::{UpdateExecutor, VertexUpdate};

        let mut vertex_updates = Vec::new();
        for info in node.updates() {
            let vertex_id = info
                .vertex_id
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("顶点ID表达式不存在或不是字面量".to_string())
                })?;

            let mut properties = HashMap::new();
            for (key, value_expr) in &info.properties {
                let value = value_expr
                    .get_expression()
                    .and_then(|e| Self::evaluate_literal(&e))
                    .ok_or_else(|| {
                        QueryError::ExecutionError(format!(
                            "属性 {} 的值表达式不存在或不是字面量",
                            key
                        ))
                    })?;
                properties.insert(key.clone(), value);
            }

            vertex_updates.push(VertexUpdate {
                vertex_id,
                properties,
                tags_to_add: None,
                tags_to_remove: None,
            });
        }

        let space_name = node
            .updates()
            .first()
            .map(|u| u.space_name.clone())
            .unwrap_or_else(|| "default".to_string());

        let is_upsert = node.updates().first().map(|u| u.is_upsert).unwrap_or(false);

        let executor = UpdateExecutor::new(
            node.id(),
            storage,
            Some(vertex_updates),
            None,
            None,
            context.expression_context().clone(),
        )
        .with_space(space_name)
        .with_insertable(is_upsert);

        Ok(ExecutorEnum::Update(executor))
    }

    /// Building the UpdateEdges executor
    pub fn build_update_edges(
        node: &UpdateEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        use crate::query::executor::data_modification::{EdgeUpdate, UpdateExecutor};

        let mut edge_updates = Vec::new();
        for info in node.updates() {
            let src = info
                .src
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("源顶点ID表达式不存在或不是字面量".to_string())
                })?;

            let dst = info
                .dst
                .get_expression()
                .and_then(|e| Self::evaluate_literal(&e))
                .ok_or_else(|| {
                    QueryError::ExecutionError("目标顶点ID表达式不存在或不是字面量".to_string())
                })?;

            let rank = info
                .rank
                .as_ref()
                .and_then(|r| r.get_expression().and_then(|e| Self::evaluate_literal(&e)))
                .and_then(|v| match v {
                    Value::Int(i) => Some(i),
                    _ => None,
                });

            let mut properties = HashMap::new();
            for (key, value_expr) in &info.properties {
                let value = value_expr
                    .get_expression()
                    .and_then(|e| Self::evaluate_literal(&e))
                    .ok_or_else(|| {
                        QueryError::ExecutionError(format!(
                            "属性 {} 的值表达式不存在或不是字面量",
                            key
                        ))
                    })?;
                properties.insert(key.clone(), value);
            }

            let edge_type = info.edge_type.clone().unwrap_or_default();

            edge_updates.push(EdgeUpdate {
                src,
                dst,
                edge_type,
                rank,
                properties,
            });
        }

        let space_name = node
            .updates()
            .first()
            .map(|u| u.space_name.clone())
            .unwrap_or_else(|| "default".to_string());

        let is_upsert = node.updates().first().map(|u| u.is_upsert).unwrap_or(false);

        let executor = UpdateExecutor::new(
            node.id(),
            storage,
            None,
            Some(edge_updates),
            None,
            context.expression_context().clone(),
        )
        .with_space(space_name)
        .with_insertable(is_upsert);

        Ok(ExecutorEnum::Update(executor))
    }
}

impl<S: StorageClient + 'static> Default for DataModificationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
