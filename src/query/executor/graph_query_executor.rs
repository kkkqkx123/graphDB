//! 图查询执行器
//!
//! 提供图查询语言（Cypher/NGQL）的执行功能
//! 支持MATCH、CREATE、DELETE等图操作语句

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value as CoreValue;
use crate::query::executor::base::{ExecutionResult, Executor, HasStorage};
use crate::query::executor::statement_executors::{DDLExecutor, UserExecutor, CypherClauseExecutor, DMLOperator, QueryExecutor, SystemExecutor};
use crate::query::parser::ast::stmt::Stmt;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 图查询执行器
///
/// 提供图查询语言（Cypher/NGQL）的执行功能
/// 支持MATCH、CREATE、DELETE等图操作语句
pub struct GraphQueryExecutor<S: StorageClient> {
    /// 执行器ID
    id: i64,
    /// 执行器名称
    name: String,
    /// 执行器描述
    description: String,
    /// 存储引擎引用
    storage: Arc<Mutex<S>>,
    /// 是否已打开
    is_open: bool,
    /// 执行统计信息
    stats: crate::query::executor::base::ExecutorStats,
}

impl<S: StorageClient> std::fmt::Debug for GraphQueryExecutor<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphQueryExecutor")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("is_open", &self.is_open)
            .field("stats", &self.stats)
            .finish()
    }
}

impl<S: StorageClient + 'static> GraphQueryExecutor<S> {
    /// 创建新的图查询执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name: "GraphQueryExecutor".to_string(),
            description: "图查询语言执行器".to_string(),
            storage,
            is_open: false,
            stats: crate::query::executor::base::ExecutorStats::new(),
        }
    }

    /// 带名称创建执行器
    pub fn with_name(id: i64, name: String, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            name,
            description: "图查询语言执行器".to_string(),
            storage,
            is_open: false,
            stats: crate::query::executor::base::ExecutorStats::new(),
        }
    }

    /// 带名称和描述创建执行器
    pub fn with_description(
        id: i64,
        name: String,
        description: String,
        storage: Arc<Mutex<S>>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            storage,
            is_open: false,
            stats: crate::query::executor::base::ExecutorStats::new(),
        }
    }

    /// 执行具体的语句
    fn execute_statement(&mut self, statement: Stmt) -> Result<ExecutionResult, DBError> {
        match statement {
            Stmt::Match(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_match(clause)
            }
            Stmt::Create(clause) => {
                let executor = DDLExecutor::new(self.id, self.storage.clone());
                executor.execute_create(clause)
            }
            Stmt::Delete(clause) => {
                let executor = DMLOperator::new(self.id, self.storage.clone());
                executor.execute_delete(clause)
            }
            Stmt::Update(clause) => {
                let executor = DMLOperator::new(self.id, self.storage.clone());
                executor.execute_update(clause)
            }
            Stmt::Query(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_query(clause)
            }
            Stmt::Go(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_go(clause)
            }
            Stmt::Fetch(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_fetch(clause)
            }
            Stmt::Lookup(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_lookup(clause)
            }
            Stmt::FindPath(clause) => {
                let executor = QueryExecutor::new(self.id, self.storage.clone());
                executor.execute_find_path(clause)
            }
            Stmt::Use(clause) => {
                let executor = SystemExecutor::new(self.id, self.storage.clone());
                executor.execute_use(clause)
            }
            Stmt::Show(clause) => {
                let executor = SystemExecutor::new(self.id, self.storage.clone());
                executor.execute_show(clause)
            }
            Stmt::Explain(clause) => {
                let executor = SystemExecutor::new(self.id, self.storage.clone());
                executor.execute_explain(clause)
            }
            Stmt::Profile(clause) => {
                let executor = SystemExecutor::new(self.id, self.storage.clone());
                executor.execute_profile(clause)
            }
            Stmt::GroupBy(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowSessions(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowQueries(_clause) => Ok(ExecutionResult::Success),
            Stmt::KillQuery(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowConfigs(_clause) => Ok(ExecutionResult::Success),
            Stmt::UpdateConfigs(_clause) => Ok(ExecutionResult::Success),
            Stmt::Assignment(clause) => self.execute_assignment(clause),
            Stmt::SetOperation(clause) => self.execute_set_operation(clause),
            Stmt::Subgraph(clause) => self.execute_subgraph(clause),
            Stmt::Insert(clause) => {
                let executor = DMLOperator::new(self.id, self.storage.clone());
                executor.execute_insert(clause)
            }
            Stmt::Merge(clause) => {
                let executor = DMLOperator::new(self.id, self.storage.clone());
                executor.execute_merge(clause)
            }
            Stmt::Unwind(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_unwind(clause)
            }
            Stmt::Return(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_return(clause)
            }
            Stmt::With(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_with(clause)
            }
            Stmt::Yield(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_yield(clause)
            }
            Stmt::Set(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_set(clause)
            }
            Stmt::Remove(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_remove(clause)
            }
            Stmt::Pipe(clause) => {
                let executor = CypherClauseExecutor::new(self.id, self.storage.clone());
                executor.execute_pipe(clause)
            }
            Stmt::Drop(clause) => {
                let executor = DDLExecutor::new(self.id, self.storage.clone());
                executor.execute_drop(clause)
            }
            Stmt::Desc(clause) => {
                let executor = DDLExecutor::new(self.id, self.storage.clone());
                executor.execute_desc(clause)
            }
            Stmt::Alter(clause) => {
                let executor = DDLExecutor::new(self.id, self.storage.clone());
                executor.execute_alter(clause)
            }
            Stmt::CreateUser(clause) => {
                let executor = UserExecutor::new(self.id, self.storage.clone());
                executor.execute_create_user(clause)
            }
            Stmt::AlterUser(clause) => {
                let executor = UserExecutor::new(self.id, self.storage.clone());
                executor.execute_alter_user(clause)
            }
            Stmt::DropUser(clause) => {
                let executor = UserExecutor::new(self.id, self.storage.clone());
                executor.execute_drop_user(clause)
            }
            Stmt::ChangePassword(clause) => {
                let executor = UserExecutor::new(self.id, self.storage.clone());
                executor.execute_change_password(clause)
            }
            Stmt::Grant(_clause) => Ok(ExecutionResult::Success),
            Stmt::Revoke(_clause) => Ok(ExecutionResult::Success),
            Stmt::DescribeUser(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowUsers(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowRoles(_clause) => Ok(ExecutionResult::Success),
            Stmt::ShowCreate(clause) => {
                let executor = SystemExecutor::new(self.id, self.storage.clone());
                executor.execute_show_create(clause)
            }
        }
    }

    fn execute_assignment(
        &mut self,
        clause: crate::query::parser::ast::stmt::AssignmentStmt,
    ) -> Result<ExecutionResult, DBError> {
        let _var_name = clause.variable.clone();

        // 执行右侧语句，暂时不存储变量
        self.execute_statement(*clause.statement)
    }

    fn execute_set_operation(
        &mut self,
        clause: crate::query::parser::ast::stmt::SetOperationStmt,
    ) -> Result<ExecutionResult, DBError> {
        use crate::core::result::Result as CoreResult;
        use crate::query::parser::ast::stmt::SetOperationType;

        let left_result = self.execute_statement(*clause.left)?;
        let right_result = self.execute_statement(*clause.right)?;

        match (&left_result, &right_result) {
            (ExecutionResult::Result(left_data), ExecutionResult::Result(right_data)) => {
                let left_rows: std::collections::HashSet<String> = left_data
                    .rows()
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect();
                let right_rows: std::collections::HashSet<String> = right_data
                    .rows()
                    .iter()
                    .map(|r| format!("{:?}", r))
                    .collect();

                let result_rows: Vec<Vec<CoreValue>> = match clause.op_type {
                    SetOperationType::Union => left_rows
                        .union(&right_rows)
                        .map(|s| vec![CoreValue::String(s.clone())])
                        .collect(),
                    SetOperationType::UnionAll => {
                        let mut all: Vec<_> = left_data
                            .rows()
                            .iter()
                            .map(|r| vec![CoreValue::String(format!("{:?}", r))])
                            .collect();
                        all.extend(
                            right_data
                                .rows()
                                .iter()
                                .map(|r| vec![CoreValue::String(format!("{:?}", r))]),
                        );
                        all
                    }
                    SetOperationType::Intersect => left_rows
                        .intersection(&right_rows)
                        .map(|s| vec![CoreValue::String(s.clone())])
                        .collect(),
                    SetOperationType::Minus => left_rows
                        .difference(&right_rows)
                        .map(|s| vec![CoreValue::String(s.clone())])
                        .collect(),
                };

                let core_result = CoreResult::from_rows(result_rows, vec!["result".to_string()]);
                Ok(ExecutionResult::from_result(core_result))
            }
            _ => Ok(ExecutionResult::Success),
        }
    }

    fn execute_subgraph(
        &mut self,
        clause: crate::query::parser::ast::stmt::SubgraphStmt,
    ) -> Result<ExecutionResult, DBError> {
        use crate::core::Value;
        use crate::query::executor::base::EdgeDirection;
        use crate::query::executor::data_processing::graph_traversal::algorithms::{
            SubgraphConfig, SubgraphExecutor,
        };
        use crate::query::executor::expression::evaluator::expression_evaluator::ExpressionEvaluator;
        use crate::query::executor::expression::DefaultExpressionContext;
        use crate::query::validator::context::ExpressionAnalysisContext;

        let storage = self.storage.clone();

        let mut context = DefaultExpressionContext::new();

        let start_vids: Vec<Value> = clause
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
                            .unwrap_or(Value::Null(crate::core::NullType::default()));
                        val
                    }
                }
            })
            .collect();

        if start_vids.is_empty() {
            return Err(DBError::Query(QueryError::ExecutionError(
                "子图查询需要至少一个起始顶点".to_string(),
            )));
        }

        // 解析步数 - 使用 IN/OUT 步数的最大值
        let steps = match clause.steps {
            crate::query::parser::ast::stmt::Steps::Fixed(n) => n,
            crate::query::parser::ast::stmt::Steps::Range { min, max } => min.max(max),
            crate::query::parser::ast::stmt::Steps::Variable(_) => {
                return Err(DBError::Query(QueryError::ExecutionError(
                    "子图查询不支持变量步数".to_string(),
                )));
            }
        };

        // 解析边方向（从 OVER 子句获取）
        let edge_direction = if let Some(ref over) = clause.over {
            match over.direction {
                crate::query::parser::ast::types::EdgeDirection::Out => EdgeDirection::Out,
                crate::query::parser::ast::types::EdgeDirection::In => EdgeDirection::In,
                crate::query::parser::ast::types::EdgeDirection::Both => EdgeDirection::Both,
            }
        } else {
            EdgeDirection::Both
        };

        // 解析边类型过滤
        let edge_types = clause.over.as_ref().map(|over| over.edge_types.clone());

        // 创建子图执行器配置
        let mut config = SubgraphConfig::new(steps).with_direction(edge_direction);

        if let Some(types) = edge_types {
            config = config.with_edge_types(types);
        }

        // 创建并执行子图查询
        let mut executor = SubgraphExecutor::new(
            self.id,
            storage,
            start_vids,
            config,
            Arc::new(ExpressionAnalysisContext::new()),
        );

        executor.open()?;

        match executor.execute() {
            Ok(execution_result) => Ok(execution_result),
            Err(e) => Err(DBError::Query(QueryError::ExecutionError(format!(
                "子图查询执行失败: {:?}",
                e
            )))),
        }
    }
}

impl<S: StorageClient> Executor<S> for GraphQueryExecutor<S> {
    fn execute(&mut self) -> DBResult<ExecutionResult> {
        Err(DBError::Query(QueryError::ExecutionError(
            "需要先设置要执行的语句".to_string(),
        )))
    }

    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn open(&mut self) -> DBResult<()> {
        self.is_open = true;
        Ok(())
    }

    fn close(&mut self) -> DBResult<()> {
        self.is_open = false;
        Ok(())
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn stats(&self) -> &crate::query::executor::base::ExecutorStats {
        &self.stats
    }

    fn stats_mut(&mut self) -> &mut crate::query::executor::base::ExecutorStats {
        &mut self.stats
    }
}

impl<S: StorageClient> HasStorage<S> for GraphQueryExecutor<S> {
    fn get_storage(&self) -> &Arc<Mutex<S>> {
        &self.storage
    }
}
