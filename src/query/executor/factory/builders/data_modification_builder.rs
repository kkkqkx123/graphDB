//! 数据修改执行器构建器
//!
//! 负责创建数据修改类型的执行器（InsertVertices, InsertEdges, Remove）

use crate::core::error::QueryError;
use crate::core::vertex_edge_path::Tag;
use crate::core::{Edge, Value, Vertex};
use crate::query::executor::base::ExecutionContext;
use crate::query::executor::data_modification::{InsertExecutor, RemoveExecutor, RemoveItem};
use crate::query::executor::executor_enum::ExecutorEnum;
use crate::query::planner::plan::core::nodes::{InsertEdgesNode, InsertVerticesNode, RemoveNode};
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

/// 数据修改执行器构建器
pub struct DataModificationBuilder<S: StorageClient + Send + 'static> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: StorageClient + Send + 'static> DataModificationBuilder<S> {
    /// 创建新的数据修改构建器
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// 构建 InsertVertices 执行器
    pub fn build_insert_vertices(
        &self,
        node: &InsertVerticesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 将节点数据转换为顶点数据
        let mut vertices = Vec::new();

        for (vid_expr, tag_values_list) in node.values() {
            // 获取顶点ID表达式
            let _vid_expr = vid_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("顶点ID表达式不存在".to_string()))?;

            // 获取标签名称
            let tag_names = node.tag_names();

            // 创建标签列表
            let mut tags = Vec::new();

            // 添加标签和属性
            for (tag_idx, tag_values) in tag_values_list.iter().enumerate() {
                if let Some(tag_name) = tag_names.get(tag_idx) {
                    // 创建标签属性映射
                    let mut tag_props = HashMap::new();

                    // 添加属性
                    if let Some(prop_names) = node.prop_names() {
                        for (prop_idx, prop_value) in tag_values.iter().enumerate() {
                            if let Some(prop_name) = prop_names.get(prop_idx) {
                                if let Some(_value_expr) = prop_value.get_expression() {
                                    // 将表达式转换为值（这里简化处理，实际应该求值）
                                    tag_props.insert(
                                        prop_name.clone(),
                                        Value::Null(crate::core::NullType::Null),
                                    );
                                }
                            }
                        }
                    }

                    // 创建标签
                    let tag = Tag::new(tag_name.clone(), tag_props);
                    tags.push(tag);
                }
            }

            // 创建顶点（使用占位ID，实际执行时会求值）
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

    /// 构建 InsertEdges 执行器
    pub fn build_insert_edges(
        &self,
        node: &InsertEdgesNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        let mut edges = Vec::new();

        for (src_expr, dst_expr, rank_expr, prop_values) in node.edges() {
            // 获取源顶点ID表达式
            let _src_expr = src_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("源顶点ID表达式不存在".to_string()))?;

            // 获取目标顶点ID表达式
            let _dst_expr = dst_expr
                .get_expression()
                .ok_or_else(|| QueryError::ExecutionError("目标顶点ID表达式不存在".to_string()))?;

            // 获取rank（可选），默认为0
            let rank = rank_expr
                .as_ref()
                .and_then(|e| e.get_expression())
                .and_then(|expr| match expr {
                    crate::core::Expression::Literal(crate::core::Value::Int(v)) => Some(v),
                    _ => None,
                })
                .unwrap_or(0);

            // 创建边属性映射
            let mut props = HashMap::new();
            let prop_names = node.prop_names();
            for (prop_idx, prop_value) in prop_values.iter().enumerate() {
                if let Some(prop_name) = prop_names.get(prop_idx) {
                    if let Some(_value_expr) = prop_value.get_expression() {
                        // 将表达式转换为值（这里简化处理，实际应该求值）
                        props.insert(prop_name.clone(), Value::Null(crate::core::NullType::Null));
                    }
                }
            }

            // 创建边（使用占位ID，实际执行时会求值）
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

    /// 构建 Remove 执行器
    pub fn build_remove(
        &self,
        node: &RemoveNode,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<ExecutorEnum<S>, QueryError> {
        // 转换remove_items
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
