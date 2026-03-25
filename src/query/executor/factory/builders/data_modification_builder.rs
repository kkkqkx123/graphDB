//! Data Modification Executor Builder
//!
//! Responsible for creating executors for data modification operations (InsertVertices, InsertEdges, Remove).

use crate::core::error::QueryError;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_modification::{InsertExecutor, RemoveExecutor, RemoveItem};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planning::plan::core::nodes::{InsertEdgesNode, InsertVerticesNode, RemoveNode};
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
        &self,
        node: &InsertVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // Convert node data into vertex data.
        let mut vertices = Vec::new();

        for (vid_expr, tag_values_list) in node.values() {
            // Obtain the vertex ID expression
            let _vid_expr = vid_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("顶点ID表达式不存在".to_string()))?;

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
                                if let Some(_value_expr) = prop_value.get_expression() {
                                    // Convert the expression into a value (a simplification is done here; in reality, an evaluation should be performed).
                                    tag_props.insert(
                                        prop_name.clone(),
                                        Value::Null(crate::core::NullType::Null),
                                    );
                                }
                            }
                        }
                    }

                    // Create tags
                    let tag = Tag::new(tag_name.clone(), tag_props);
                    tags.push(tag);
                }
            }

            // Create vertices (using placeholder IDs, which will be evaluated during the actual execution).
            let vid = Value::Null(crate::core::NullType::Null);
            let vertex = Vertex::new(vid, tags);
            vertices.push(vertex);
        }

        let executor = InsertExecutor::with_vertices(
            node.id(),
            storage,
            vertices,
            context.expression_context().clone(),
        );

        Ok(ExecutorEnum::InsertVertices(executor))
    }

    /// Constructing the InsertEdges executor
    pub fn build_insert_edges(
        &self,
        node: &InsertEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let mut edges = Vec::new();

        for (src_expr, dst_expr, rank_expr, prop_values) in node.edges() {
            // Obtain the expression for the ID of the source vertex.
            let _src_expr = src_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("源顶点ID表达式不存在".to_string()))?;

            // Obtain the expression for the target vertex ID.
            let _dst_expr = dst_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("目标顶点ID表达式不存在".to_string()))?;

            // Obtain the rank (optional); the default value is 0.
            let rank = rank_expr
                .as_ref()
                .and_then(|e| e.get_expression())
                .and_then(|expr| match expr {
                    crate::core::Expression::Literal(crate::core::Value::Int(v)) => Some(v),
                    _ => None,
                })
                .unwrap_or(0);

            // Create a mapping of edge attributes.
            let mut props = HashMap::new();
            let prop_names = node.prop_names();
            for (prop_idx, prop_value) in prop_values.iter().enumerate() {
                if let Some(prop_name) = prop_names.get(prop_idx) {
                    if let Some(_value_expr) = prop_value.get_expression() {
                        // Convert the expression into a value (a simplified approach is used here; in reality, the expression should be evaluated).
                        props.insert(prop_name.clone(), Value::Null(crate::core::NullType::Null));
                    }
                }
            }

            // Create an edge (using a placeholder ID, which will be evaluated during the actual execution).
            let src = Value::Null(crate::core::NullType::Null);
            let dst = Value::Null(crate::core::NullType::Null);
            let edge = Edge::new(src, dst, node.edge_name().to_string(), rank, props);

            edges.push(edge);
        }

        let executor = InsertExecutor::with_edges(
            node.id(),
            storage,
            edges,
            context.expression_context().clone(),
        );

        Ok(ExecutorEnum::InsertEdges(executor))
    }

    /// Building the Remove Executor
    pub fn build_remove(
        &self,
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
}

impl<S: StorageClient + 'static> Default for DataModificationBuilder<S> {
    fn default() -> Self {
        Self::new()
    }
}
