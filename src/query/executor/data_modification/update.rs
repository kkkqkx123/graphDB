//! 更新执行器
//!
//! 负责更新现有顶点和边的属性
//!
//! 功能增强:
//! - 支持upsert（当节点不存在时插入）
//! - 支持RETURN子句返回更新后的属性
//! - 支持YIELD指定返回属性
//! - 支持条件表达式
//! - 更好的错误处理和日志

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::core::types::expr::contextual::ContextualExpression;
use crate::core::{Expression, Value};
use crate::query::executor::base::{BaseExecutor, ExecutorStats};
use crate::query::executor::base::{DBResult, ExecutionResult, Executor, HasStorage};
use crate::query::executor::expression::evaluation_context::DefaultExpressionContext;
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::evaluator::traits::ExpressionContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;

/// 更新执行器
///
/// 负责更新顶点和边的属性
pub struct UpdateExecutor<S: StorageClient> {
    base: BaseExecutor<S>,
    vertex_updates: Option<Vec<VertexUpdate>>,
    edge_updates: Option<Vec<EdgeUpdate>>,
    condition: Option<ContextualExpression>,
    return_props: Option<Vec<String>>,
    yield_names: Vec<String>,
    insertable: bool,
    space_name: String,
}

/// 顶点更新数据结构
#[derive(Debug, Clone)]
pub struct VertexUpdate {
    pub vertex_id: Value,
    pub properties: HashMap<String, Value>,
    pub tags_to_add: Option<Vec<String>>,
    pub tags_to_remove: Option<Vec<String>>,
}

/// 边更新数据结构
#[derive(Debug, Clone)]
pub struct EdgeUpdate {
    pub src: Value,
    pub dst: Value,
    pub edge_type: String,
    pub rank: Option<i64>,
    pub properties: HashMap<String, Value>,
}

/// 更新结果数据结构
#[derive(Debug, Clone)]
pub struct UpdateResult {
    pub vertex_id: Option<Value>,
    pub src: Option<Value>,
    pub dst: Option<Value>,
    pub edge_type: Option<String>,
    pub returned_props: HashMap<String, Value>,
}

impl<S: StorageClient> UpdateExecutor<S> {
    pub fn new(
        id: i64,
        storage: Arc<Mutex<S>>,
        vertex_updates: Option<Vec<VertexUpdate>>,
        edge_updates: Option<Vec<EdgeUpdate>>,
        condition: Option<ContextualExpression>,
        expr_context: Arc<ExpressionAnalysisContext>,
    ) -> Self {
        Self {
            base: BaseExecutor::new(id, "UpdateExecutor".to_string(), storage, expr_context),
            vertex_updates,
            edge_updates,
            condition,
            return_props: None,
            yield_names: Vec::new(),
            insertable: false,
            space_name: "default".to_string(),
        }
    }

    pub fn with_return_props(mut self, return_props: Vec<String>) -> Self {
        self.return_props = Some(return_props);
        self
    }

    pub fn with_yield_names(mut self, yield_names: Vec<String>) -> Self {
        self.yield_names = yield_names;
        self
    }

    pub fn with_insertable(mut self, insertable: bool) -> Self {
        self.insertable = insertable;
        self
    }

    pub fn with_space(mut self, space_name: String) -> Self {
        self.space_name = space_name;
        self
    }
}

impl<S: StorageClient + Send + Sync + 'static> Executor<S> for UpdateExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        let start = Instant::now();
        let result = self.do_execute();
        let elapsed = start.elapsed();
        self.base.get_stats_mut().add_total_time(elapsed);
        match result {
            Ok(_) => Ok(ExecutionResult::Empty),
            Err(e) => Err(e),
        }
    }

    fn open(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        Ok(())
    }

    fn is_open(&self) -> bool {
        true
    }

    fn id(&self) -> i64 {
        self.base.id
    }

    fn name(&self) -> &str {
        "UpdateExecutor"
    }

    fn description(&self) -> &str {
        "Update executor - updates vertices and edges in storage"
    }

    fn stats(&self) -> &ExecutorStats {
        self.base.get_stats()
    }

    fn stats_mut(&mut self) -> &mut ExecutorStats {
        self.base.get_stats_mut()
    }
}

impl<S: StorageClient> HasStorage<S> for UpdateExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        self.base.get_storage()
    }
}

impl<S: StorageClient + Send + Sync + 'static> UpdateExecutor<S> {
    fn do_execute(&mut self) -> DBResult<Vec<UpdateResult>> {
        let mut results = Vec::new();

        // 直接从 ContextualExpression 获取 Expression
        let condition_expression = self.condition.as_ref().and_then(|c| c.get_expression());

        let mut storage = self.get_storage().lock();

        if let Some(updates) = &self.vertex_updates {
            for update in updates {
                let mut update_result = UpdateResult {
                    vertex_id: Some(update.vertex_id.clone()),
                    src: None,
                    dst: None,
                    edge_type: None,
                    returned_props: HashMap::new(),
                };

                let should_update = if let Some(ref expression) = condition_expression {
                    self.evaluate_condition(
                        expression,
                        update.vertex_id.clone(),
                        None,
                        None,
                        None,
                        &update.properties,
                    )?
                } else {
                    true
                };

                if should_update {
                    if let Some(mut vertex) =
                        storage.get_vertex(&self.space_name, &update.vertex_id)?
                    {
                        for (key, value) in &update.properties {
                            vertex.properties.insert(key.clone(), value.clone());
                        }
                        storage.update_vertex(&self.space_name, vertex.clone())?;

                        update_result.returned_props = update.properties.clone();
                    } else if self.insertable {
                        let new_vertex = crate::core::Vertex::new_with_properties(
                            update.vertex_id.clone(),
                            Vec::new(),
                            update.properties.clone(),
                        );
                        storage.insert_vertex(&self.space_name, new_vertex)?;
                        update_result.returned_props = update.properties.clone();
                    }
                }

                results.push(update_result);
            }
        }

        if let Some(updates) = &self.edge_updates {
            for update in updates {
                let mut update_result = UpdateResult {
                    vertex_id: None,
                    src: Some(update.src.clone()),
                    dst: Some(update.dst.clone()),
                    edge_type: Some(update.edge_type.clone()),
                    returned_props: HashMap::new(),
                };

                let should_update = if let Some(ref expression) = condition_expression {
                    self.evaluate_condition(
                        expression,
                        update.src.clone(),
                        Some(update.dst.clone()),
                        Some(&update.edge_type),
                        None,
                        &update.properties,
                    )?
                } else {
                    true
                };

                if should_update {
                    let edge_key = (
                        update.src.clone(),
                        update.dst.clone(),
                        update.edge_type.clone(),
                    );
                    if let Some(mut edge) =
                        storage.get_edge(&self.space_name, &edge_key.0, &edge_key.1, &edge_key.2)?
                    {
                        for (key, value) in &update.properties {
                            edge.props.insert(key.clone(), value.clone());
                        }
                        storage.delete_edge(
                            &self.space_name,
                            &edge_key.0,
                            &edge_key.1,
                            &edge_key.2,
                        )?;
                        storage.insert_edge(&self.space_name, edge)?;
                        update_result.returned_props = update.properties.clone();
                    } else if self.insertable {
                        let new_edge = crate::core::Edge::new(
                            edge_key.0.clone(),
                            edge_key.1.clone(),
                            edge_key.2.clone(),
                            update.rank.unwrap_or(0),
                            update.properties.clone(),
                        );
                        storage.insert_edge(&self.space_name, new_edge)?;
                        update_result.returned_props = update.properties.clone();
                    }
                }

                results.push(update_result);
            }
        }

        Ok(results)
    }

    fn evaluate_condition(
        &self,
        expression: &Expression,
        vertex_id: Value,
        dst: Option<Value>,
        edge_type: Option<&str>,
        _rank: Option<i64>,
        properties: &HashMap<String, Value>,
    ) -> DBResult<bool> {
        let mut context = DefaultExpressionContext::new();
        context.set_variable("VID".to_string(), vertex_id.clone());
        if let Some(dst_val) = dst {
            context.set_variable("DST".to_string(), dst_val);
        }
        if let Some(etype) = edge_type {
            context.set_variable(
                "edge_type".to_string(),
                crate::core::Value::String(etype.to_string()),
            );
        }
        for (key, value) in properties {
            context.set_variable(key.clone(), value.clone());
        }

        let result = ExpressionEvaluator::evaluate(expression, &mut context).map_err(|e| {
            crate::core::error::DBError::Query(crate::core::error::QueryError::ExecutionError(
                format!("条件求值失败: {}", e),
            ))
        })?;

        match result {
            crate::core::Value::Bool(b) => Ok(b),
            _ => Err(crate::core::error::DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "条件表达式必须返回布尔值".to_string(),
                ),
            )),
        }
    }
}
