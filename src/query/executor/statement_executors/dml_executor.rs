use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::vertex_edge_path::Tag;
use crate::core::Edge;
use crate::core::Vertex;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::data_modification::{DeleteExecutor, EdgeUpdate, InsertExecutor, UpdateExecutor, VertexUpdate};
use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
use crate::query::executor::expression::DefaultExpressionContext;
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::{DeleteStmt, DeleteTarget, InsertStmt, InsertTarget, MergeStmt, UpdateStmt, UpdateTarget};
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::merge_planner::MergePlanner;
use crate::query::parser::ast::Ast;
use crate::query::QueryContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

pub struct DMLOperator<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> DMLOperator<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    pub fn execute_delete(&self, clause: DeleteStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::data_modification::DeleteTagExecutor;

        match clause.target {
            DeleteTarget::Vertices(vertex_exprs) => {
                let mut vertex_ids = Vec::new();
                for ctx_expr in vertex_exprs {
                    let expr = ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("表达式不存在".to_string()))
                    })?;
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&expr, &mut context).map_err(|e| {
                        DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e)))
                    })?;
                    vertex_ids.push(vid);
                }

                let mut executor = DeleteExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_ids),
                    None,
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                executor.execute()
            }
            DeleteTarget::Edges { edge_type, edges } => {
                let edge_type_str = edge_type.unwrap_or_default();
                let mut edge_ids = Vec::new();
                for (src_ctx_expr, dst_ctx_expr, _rank_ctx_expr) in edges {
                    let src_expr = src_ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("表达式不存在".to_string()))
                    })?;
                    let mut src_context = DefaultExpressionContext::new();
                    let src = ExpressionEvaluator::evaluate(&src_expr, &mut src_context).map_err(
                        |e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "源顶点ID求值失败: {}",
                                e
                            )))
                        },
                    )?;

                    let dst_expr = dst_ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("表达式不存在".to_string()))
                    })?;
                    let mut dst_context = DefaultExpressionContext::new();
                    let dst = ExpressionEvaluator::evaluate(&dst_expr, &mut dst_context).map_err(
                        |e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "目标顶点ID求值失败: {}",
                                e
                            )))
                        },
                    )?;

                    edge_ids.push((src, dst, edge_type_str.clone()));
                }

                let mut executor = DeleteExecutor::new(
                    self.id,
                    self.storage.clone(),
                    None,
                    Some(edge_ids),
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                executor.execute()
            }
            DeleteTarget::Tags {
                tag_names,
                vertex_ids: vertex_id_exprs,
                is_all_tags,
            } => {
                let mut vertex_ids = Vec::new();
                for ctx_expr in vertex_id_exprs {
                    let expr = ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("表达式不存在".to_string()))
                    })?;
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&expr, &mut context).map_err(|e| {
                        DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e)))
                    })?;
                    vertex_ids.push(vid);
                }

                let executor = DeleteTagExecutor::new(
                    self.id,
                    self.storage.clone(),
                    tag_names,
                    vertex_ids,
                    Arc::new(ExpressionAnalysisContext::new()),
                )
                .with_space("default".to_string());

                let mut executor = if is_all_tags {
                    executor.delete_all_tags()
                } else {
                    executor
                };

                Executor::open(&mut executor)?;
                executor.execute()
            }
            DeleteTarget::Index(index_name) => Err(DBError::Query(QueryError::ExecutionError(
                format!("DELETE INDEX {} 未实现", index_name),
            ))),
        }
    }

    pub fn execute_update(&self, clause: UpdateStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match clause.target {
            UpdateTarget::Vertex(vid_expr) => {
                let mut context = DefaultExpressionContext::new();
                let vid_expr_inner = vid_expr.expression().ok_or_else(|| {
                    DBError::Query(QueryError::ExecutionError("顶点ID表达式无效".to_string()))
                })?;
                let vid = ExpressionEvaluator::evaluate(vid_expr_inner.inner(), &mut context)
                    .map_err(|e| {
                        DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e)))
                    })?;

                let mut properties = HashMap::new();
                for assignment in &clause.set_clause.assignments {
                    let mut prop_context = DefaultExpressionContext::new();
                    let value_expr_inner = assignment.value.expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("属性值表达式无效".to_string()))
                    })?;
                    let value =
                        ExpressionEvaluator::evaluate(value_expr_inner.inner(), &mut prop_context)
                            .map_err(|e| {
                                DBError::Query(QueryError::ExecutionError(format!(
                                    "属性值求值失败: {}",
                                    e
                                )))
                            })?;
                    properties.insert(assignment.property.clone(), value);
                }

                let vertex_updates = vec![VertexUpdate {
                    vertex_id: vid,
                    properties,
                    tags_to_add: None,
                    tags_to_remove: None,
                }];

                let mut executor = UpdateExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_updates),
                    None,
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                )
                .with_insertable(false)
                .with_space("default".to_string());

                Executor::open(&mut executor)?;
                executor.execute()
            }
            UpdateTarget::Edge {
                src,
                dst,
                edge_type,
                rank,
            } => {
                let edge_type_str = edge_type.unwrap_or_default();
                let mut src_context = DefaultExpressionContext::new();
                let src_expr_inner = src.expression().ok_or_else(|| {
                    DBError::Query(QueryError::ExecutionError("源顶点ID表达式无效".to_string()))
                })?;
                let src_val =
                    ExpressionEvaluator::evaluate(src_expr_inner.inner(), &mut src_context)
                        .map_err(|e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "源顶点ID求值失败: {}",
                                e
                            )))
                        })?;

                let mut dst_context = DefaultExpressionContext::new();
                let dst_expr_inner = dst.expression().ok_or_else(|| {
                    DBError::Query(QueryError::ExecutionError(
                        "目标顶点ID表达式无效".to_string(),
                    ))
                })?;
                let dst_val =
                    ExpressionEvaluator::evaluate(dst_expr_inner.inner(), &mut dst_context)
                        .map_err(|e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "目标顶点ID求值失败: {}",
                                e
                            )))
                        })?;

                let rank_val = match rank {
                    Some(ref r) => {
                        let mut rank_context = DefaultExpressionContext::new();
                        let rank_expr_inner = r.expression().ok_or_else(|| {
                            DBError::Query(QueryError::ExecutionError("rank表达式无效".to_string()))
                        })?;
                        let rank_value = ExpressionEvaluator::evaluate(rank_expr_inner.inner(), &mut rank_context)
                            .map_err(|e| {
                                DBError::Query(QueryError::ExecutionError(format!(
                                    "rank求值失败: {}",
                                    e
                                )))
                            })?;
                        match rank_value {
                            crate::core::Value::Int(i) => Some(i),
                            _ => {
                                return Err(DBError::Query(QueryError::ExecutionError(
                                    "rank必须是整数".to_string(),
                                )))
                            }
                        }
                    }
                    None => None,
                };

                let mut properties = HashMap::new();
                for assignment in &clause.set_clause.assignments {
                    let mut prop_context = DefaultExpressionContext::new();
                    let value_expr_inner = assignment.value.expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("属性值表达式无效".to_string()))
                    })?;
                    let value =
                        ExpressionEvaluator::evaluate(value_expr_inner.inner(), &mut prop_context)
                            .map_err(|e| {
                                DBError::Query(QueryError::ExecutionError(format!(
                                    "属性值求值失败: {}",
                                    e
                                )))
                            })?;
                    properties.insert(assignment.property.clone(), value);
                }

                let edge_updates = vec![EdgeUpdate {
                    src: src_val,
                    dst: dst_val,
                    edge_type: edge_type_str,
                    rank: rank_val,
                    properties,
                }];

                let mut executor = UpdateExecutor::new(
                    self.id,
                    self.storage.clone(),
                    None,
                    Some(edge_updates),
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                )
                .with_insertable(false)
                .with_space("default".to_string());

                Executor::open(&mut executor)?;
                executor.execute()
            }
            UpdateTarget::Tag(tag_name) => Err(DBError::Query(QueryError::ExecutionError(
                format!("UPDATE TAG {} 未实现", tag_name),
            ))),
            UpdateTarget::TagOnVertex { vid: _, tag_name } => Err(DBError::Query(QueryError::ExecutionError(
                format!("UPDATE VERTEX ON TAG {} 未实现", tag_name),
            ))),
        }
    }

    pub fn execute_insert(&self, clause: InsertStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match clause.target {
            InsertTarget::Vertices { tags, values } => {
                let mut vertices = Vec::new();

                let tag_spec = tags.first().ok_or_else(|| {
                    DBError::Query(QueryError::ExecutionError(
                        "INSERT VERTEX 必须指定至少一个 Tag".to_string(),
                    ))
                })?;
                let tag_name = &tag_spec.tag_name;
                let prop_names = &tag_spec.prop_names;

                for row in values {
                    let mut context = DefaultExpressionContext::new();
                    let vid_expr = row.vid.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("顶点ID表达式不存在".to_string()))
                    })?;
                    let vid =
                        ExpressionEvaluator::evaluate(&vid_expr, &mut context).map_err(|e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "表达式求值失败: {}",
                                e
                            )))
                        })?;

                    let prop_values = row.tag_values.into_iter().next().unwrap_or_default();

                    let mut properties = HashMap::new();
                    for (i, prop_name) in prop_names.iter().enumerate() {
                        if i < prop_values.len() {
                            let mut prop_context = DefaultExpressionContext::new();
                            let prop_expr = prop_values[i].get_expression().ok_or_else(|| {
                                DBError::Query(QueryError::ExecutionError(
                                    "属性表达式不存在".to_string(),
                                ))
                            })?;
                            let prop_value =
                                ExpressionEvaluator::evaluate(&prop_expr, &mut prop_context)
                                    .map_err(|e| {
                                        DBError::Query(QueryError::ExecutionError(format!(
                                            "属性值求值失败: {}",
                                            e
                                        )))
                                    })?;
                            properties.insert(prop_name.clone(), prop_value);
                        }
                    }

                    let tag = Tag::new(tag_name.clone(), HashMap::new());
                    let vertex = Vertex::new_with_properties(vid, vec![tag], properties);
                    vertices.push(vertex);
                }

                let expr_context = Arc::new(ExpressionAnalysisContext::new());
                let mut executor = if clause.if_not_exists {
                    InsertExecutor::with_vertices_if_not_exists(
                        self.id,
                        self.storage.clone(),
                        vertices,
                        expr_context.clone(),
                    )
                } else {
                    InsertExecutor::with_vertices(
                        self.id,
                        self.storage.clone(),
                        vertices,
                        expr_context,
                    )
                };

                Executor::open(&mut executor)?;
                executor.execute()
            }
            InsertTarget::Edge {
                edge_name,
                prop_names,
                edges,
            } => {
                let mut edge_list = Vec::new();

                for (src_ctx_expr, dst_ctx_expr, rank_ctx_expr, prop_values) in edges {
                    let mut src_context = DefaultExpressionContext::new();
                    let src_expr = src_ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("源顶点ID表达式不存在".to_string()))
                    })?;
                    let src = ExpressionEvaluator::evaluate(&src_expr, &mut src_context)
                        .map_err(|e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "源顶点ID求值失败: {}",
                                e
                            )))
                        })?;

                    let mut dst_context = DefaultExpressionContext::new();
                    let dst_expr = dst_ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("目标顶点ID表达式不存在".to_string()))
                    })?;
                    let dst = ExpressionEvaluator::evaluate(&dst_expr, &mut dst_context)
                        .map_err(|e| {
                            DBError::Query(QueryError::ExecutionError(format!(
                                "目标顶点ID求值失败: {}",
                                e
                            )))
                        })?;

                    let rank = match rank_ctx_expr {
                        Some(ref r) => {
                            let rank_expr = r.get_expression().ok_or_else(|| {
                                DBError::Query(QueryError::ExecutionError("rank表达式不存在".to_string()))
                            })?;
                            let mut rank_context = DefaultExpressionContext::new();
                            let rank_val = ExpressionEvaluator::evaluate(&rank_expr, &mut rank_context).map_err(
                                |e| {
                                    DBError::Query(QueryError::ExecutionError(format!(
                                        "rank求值失败: {}",
                                        e
                                    )))
                                },
                            )?;
                            match rank_val {
                                crate::core::Value::Int(i) => i,
                                _ => {
                                    return Err(DBError::Query(QueryError::ExecutionError(
                                        "rank必须是整数".to_string(),
                                    )))
                                }
                            }
                        }
                        None => 0,
                    };

                    let mut properties = HashMap::new();
                    for (i, prop_name) in prop_names.iter().enumerate() {
                        if i < prop_values.len() {
                            let mut prop_context = DefaultExpressionContext::new();
                            let prop_expr = prop_values[i].get_expression().ok_or_else(|| {
                                DBError::Query(QueryError::ExecutionError(
                                    "属性表达式不存在".to_string(),
                                ))
                            })?;
                            let prop_value =
                                ExpressionEvaluator::evaluate(&prop_expr, &mut prop_context)
                                    .map_err(|e| {
                                        DBError::Query(QueryError::ExecutionError(format!(
                                            "属性值求值失败: {}",
                                            e
                                        )))
                                    })?;
                            properties.insert(prop_name.clone(), prop_value);
                        }
                    }

                    let edge = Edge::new(src, dst, edge_name.clone(), rank, properties);
                    edge_list.push(edge);
                }

                let expr_context = Arc::new(ExpressionAnalysisContext::new());
                let mut executor = if clause.if_not_exists {
                    InsertExecutor::with_edges_if_not_exists(
                        self.id,
                        self.storage.clone(),
                        edge_list,
                        expr_context.clone(),
                    )
                } else {
                    InsertExecutor::with_edges(
                        self.id,
                        self.storage.clone(),
                        edge_list,
                        expr_context,
                    )
                };

                Executor::open(&mut executor)?;
                executor.execute()
            }
        }
    }

    pub fn execute_merge(&self, clause: MergeStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(crate::query::parser::ast::stmt::Stmt::Merge(clause), ctx));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = MergePlanner::new();
        let plan = planner
            .transform(&validated, qctx)
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let root_node = plan
            .root()
            .as_ref()
            .ok_or_else(|| DBError::Query(QueryError::ExecutionError("执行计划为空".to_string())))?
            .clone();

        let mut executor_factory = ExecutorFactory::with_storage(self.storage.clone());
        let mut executor = executor_factory
            .create_executor(&root_node, self.storage.clone(), &Default::default())
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Executor::open(&mut executor)?;
        let result = Executor::execute(&mut executor)?;
        Executor::close(&mut executor)?;
        Ok(result)
    }
}
