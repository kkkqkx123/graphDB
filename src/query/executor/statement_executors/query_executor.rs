//! 查询执行器
//!
//! 处理图查询操作，包括 MATCH、GO、FETCH、LOOKUP、FIND PATH 等

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::{
    FetchStmt, FindPathStmt, GoStmt, LookupStmt, MatchStmt, QueryStmt,
};
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::go_planner::GoPlanner;
use crate::query::planner::statements::match_statement_planner::MatchStatementPlanner;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::query::QueryContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 查询执行器
///
/// 处理图查询操作，包括 MATCH、GO、FETCH、LOOKUP、FIND PATH 等
pub struct QueryExecutor<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> QueryExecutor<S> {
    /// 创建新的查询执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    /// 执行 MATCH 查询
    pub fn execute_match(&self, clause: MatchStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        // 创建验证后的语句
        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(crate::query::parser::ast::Ast::new(
            crate::query::parser::ast::stmt::Stmt::Match(clause),
            ctx,
        ));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = MatchStatementPlanner::new();
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

        executor
            .open()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let result = executor
            .execute()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .close()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Ok(result)
    }

    /// 执行 GO 查询
    pub fn execute_go(&self, clause: GoStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::parser::ast::Ast;

        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(
            crate::query::parser::ast::stmt::Stmt::Go(clause),
            ctx,
        ));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = GoPlanner::new();
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

        executor
            .open()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        let result = executor
            .execute()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        executor
            .close()
            .map_err(|e| DBError::Query(QueryError::ExecutionError(e.to_string())))?;

        Ok(result)
    }

    /// 执行 FETCH 查询
    pub fn execute_fetch(&self, clause: FetchStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::data_access::{GetEdgesExecutor, GetVerticesExecutor};
        use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::query::executor::expression::DefaultExpressionContext;
        use crate::query::parser::ast::stmt::FetchTarget;

        match clause.target {
            FetchTarget::Vertices { ids, properties: _ } => {
                let mut vertex_ids = Vec::new();
                for ctx_expr in ids {
                    let expr = ctx_expr.get_expression().ok_or_else(|| {
                        DBError::Query(QueryError::ExecutionError("表达式不存在".to_string()))
                    })?;
                    let mut context = DefaultExpressionContext::new();
                    let vid = ExpressionEvaluator::evaluate(&expr, &mut context).map_err(|e| {
                        DBError::Query(QueryError::ExecutionError(format!("顶点ID求值失败: {}", e)))
                    })?;
                    vertex_ids.push(vid);
                }

                let mut executor = GetVerticesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(vertex_ids),
                    None,
                    None,
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                executor.open()?;
                executor.execute()
            }
            FetchTarget::Edges {
                src: _,
                dst: _,
                edge_type,
                rank: _,
                properties: _,
            } => {
                let mut executor = GetEdgesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Some(edge_type),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                executor.open()?;
                executor.execute()
            }
        }
    }

    /// 执行 LOOKUP 查询
    pub fn execute_lookup(&self, clause: LookupStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::data_access::LookupIndexExecutor;
        use crate::query::parser::ast::stmt::LookupTarget;

        match clause.target {
            LookupTarget::Tag(tag_name) => {
                let mut executor = LookupIndexExecutor::new(
                    self.id,
                    self.storage.clone(),
                    format!("idx_{}", tag_name),
                    None,
                    true,
                    None,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                executor.open()?;
                executor.execute()
            }
            LookupTarget::Edge(edge_name) => Err(DBError::Query(QueryError::ExecutionError(
                format!("LOOKUP ON EDGE {} 未实现", edge_name),
            ))),
        }
    }

    /// 执行 FIND PATH 查询
    pub fn execute_find_path(&self, clause: FindPathStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::base::EdgeDirection;
        use crate::query::executor::data_processing::graph_traversal::AllPathsExecutor;
        use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::query::executor::expression::DefaultExpressionContext;

        let storage = self.storage.clone();

        let mut context = DefaultExpressionContext::new();

        let left_start_ids: Vec<Value> = clause
            .from
            .vertices
            .iter()
            .map(|ctx_expr| {
                let expr = ctx_expr.get_expression().unwrap_or_else(|| {
                    crate::core::types::expression::Expression::Literal(Value::Null(
                        crate::core::NullType::default(),
                    ))
                });
                match expr {
                    crate::core::types::expression::Expression::Literal(Value::Int(n)) => {
                        Value::Int(n)
                    }
                    crate::core::types::expression::Expression::Literal(Value::String(s)) => {
                        Value::String(s)
                    }
                    _ => {
                        let val = ExpressionEvaluator::evaluate(&expr, &mut context)
                            .unwrap_or_else(|_| Value::Null(crate::core::NullType::default()));
                        val
                    }
                }
            })
            .collect();

        let to_expr = clause.to.get_expression().unwrap_or_else(|| {
            crate::core::types::expression::Expression::Literal(Value::Null(
                crate::core::NullType::default(),
            ))
        });
        let right_start_ids: Vec<Value> = vec![match to_expr {
            crate::core::types::expression::Expression::Literal(Value::Int(n)) => Value::Int(n),
            crate::core::types::expression::Expression::Literal(Value::String(s)) => {
                Value::String(s)
            }
            _ => {
                let val = ExpressionEvaluator::evaluate(&to_expr, &mut context)
                    .unwrap_or_else(|_| Value::Null(crate::core::NullType::default()));
                val
            }
        }];

        // 解析边方向
        let edge_direction = if let Some(ref over) = clause.over {
            match over.direction {
                crate::query::parser::ast::types::EdgeDirection::Out => EdgeDirection::Out,
                crate::query::parser::ast::types::EdgeDirection::In => EdgeDirection::In,
                crate::query::parser::ast::types::EdgeDirection::Both => EdgeDirection::Both,
            }
        } else {
            EdgeDirection::Both
        };

        // 解析边类型
        let edge_types = clause.over.as_ref().map(|over| over.edge_types.clone());

        // 解析最大步数
        let max_steps = clause.max_steps.unwrap_or(5);

        // 解析 limit 和 offset
        let limit = clause.limit.unwrap_or(std::usize::MAX);
        let offset = clause.offset.unwrap_or(0);

        // 创建执行器
        let mut executor = AllPathsExecutor::new(
            0,
            storage,
            left_start_ids,
            right_start_ids,
            edge_direction,
            edge_types,
            max_steps,
            Arc::new(ExpressionAnalysisContext::new()),
        )
        .with_config(
            false, // with_prop
            limit,
            offset,
        )
        .with_loop(clause.with_loop);

        // 执行查询
        match executor.execute() {
            Ok(_paths) => {
                // 转换为 ExecutionResult
                let core_result = crate::core::result::Result::empty(vec!["path".to_string()]);
                let result = ExecutionResult::from_result(core_result);
                Ok(result)
            }
            Err(e) => Err(DBError::Query(QueryError::ExecutionError(format!(
                "FIND PATH执行失败: {:?}",
                e
            )))),
        }
    }

    /// 执行复合查询
    pub fn execute_query(&self, clause: QueryStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let mut result = ExecutionResult::Success;

        for stmt in clause.statements {
            result = match stmt {
                crate::query::parser::ast::Stmt::Match(match_stmt) => self.execute_match(match_stmt)?,
                crate::query::parser::ast::Stmt::Go(go_stmt) => self.execute_go(go_stmt)?,
                crate::query::parser::ast::Stmt::Fetch(fetch_stmt) => self.execute_fetch(fetch_stmt)?,
                crate::query::parser::ast::Stmt::Lookup(lookup_stmt) => self.execute_lookup(lookup_stmt)?,
                crate::query::parser::ast::Stmt::FindPath(find_path_stmt) => self.execute_find_path(find_path_stmt)?,
                _ => {
                    return Err(DBError::Query(QueryError::ExecutionError(
                        format!("不支持的语句类型: {:?}", stmt.kind()),
                    )))
                }
            };
        }

        Ok(result)
    }
}
