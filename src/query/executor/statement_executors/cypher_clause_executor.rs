use crate::core::error::{DBError, DBResult, QueryError};
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::executor::factory::ExecutorFactory;
use crate::query::parser::ast::stmt::{PipeStmt, RemoveStmt, ReturnStmt, SetStmt, UnwindStmt, WithStmt, YieldStmt};
use crate::query::planner::planner::{Planner, ValidatedStatement};
use crate::query::planner::statements::remove_planner::RemovePlanner;
use crate::query::planner::statements::return_planner::ReturnPlanner;
use crate::query::planner::statements::with_planner::WithPlanner;
use crate::query::planner::statements::yield_planner::YieldPlanner;
use crate::query::parser::ast::Ast;
use crate::query::QueryContext;
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::query::validator::ValidationInfo;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

pub struct CypherClauseExecutor<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> CypherClauseExecutor<S> {
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    pub fn execute_unwind(&self, clause: UnwindStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::result_processing::transformations::unwind::UnwindExecutor;

        let expr = clause.expression.get_expression().ok_or_else(|| {
            DBError::Query(QueryError::ExecutionError("UNWIND表达式不存在".to_string()))
        })?;

        let mut executor = UnwindExecutor::new(
            self.id,
            self.storage.clone(),
            "_input".to_string(),
            expr,
            vec![clause.variable.clone()],
            false,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        executor.execute()
    }

    #[allow(dead_code)]
    pub fn execute_return(&self, clause: ReturnStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(crate::query::parser::ast::stmt::Stmt::Return(clause), ctx));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = ReturnPlanner::new();
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

    #[allow(dead_code)]
    pub fn execute_with(&self, clause: WithStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(crate::query::parser::ast::stmt::Stmt::With(clause), ctx));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = WithPlanner::new();
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

    #[allow(dead_code)]
    pub fn execute_yield(&self, clause: YieldStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(crate::query::parser::ast::stmt::Stmt::Yield(clause), ctx));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = YieldPlanner::new();
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

    pub fn execute_set(&self, clause: SetStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::result_processing::transformations::assign::AssignExecutor;

        let mut assignments = Vec::new();
        for assignment in clause.assignments {
            let expr = assignment.value.get_expression().ok_or_else(|| {
                DBError::Query(QueryError::ExecutionError("SET表达式不存在".to_string()))
            })?;
            assignments.push((assignment.property, expr));
        }

        let mut executor = AssignExecutor::new(
            self.id,
            self.storage.clone(),
            assignments,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        executor.execute()
    }

    pub fn execute_remove(&self, clause: RemoveStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let qctx = Arc::new(QueryContext::default());

        let validation_info = ValidationInfo::new();
        let ctx = Arc::new(ExpressionAnalysisContext::new());
        let ast = Arc::new(Ast::new(crate::query::parser::ast::stmt::Stmt::Remove(clause), ctx));
        let validated = ValidatedStatement::new(ast, validation_info);

        let mut planner = RemovePlanner::new();
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

    #[allow(dead_code)]
    pub fn execute_pipe(&self, clause: PipeStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        let left_result = self.execute_statement(*clause.left.clone())?;

        match left_result {
            ExecutionResult::Values(values) => {
                if values.is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::Result(data_set) => {
                if data_set.rows().is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::Empty => {
                return Ok(ExecutionResult::Empty);
            }
            ExecutionResult::Success => {}
            ExecutionResult::Vertices(vertices) => {
                if vertices.is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::Edges(edges) => {
                if edges.is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::DataSet(data_set) => {
                if data_set.rows.is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::Count(_) => {}
            ExecutionResult::Paths(paths) => {
                if paths.is_empty() {
                    return Ok(ExecutionResult::Empty);
                }
            }
            ExecutionResult::Error(_) => {
                return Ok(ExecutionResult::Empty);
            }
        }

        let right_result = self.execute_statement(*clause.right)?;

        Ok(right_result)
    }

    fn execute_statement(&self, statement: crate::query::parser::ast::stmt::Stmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        match statement {
            crate::query::parser::ast::stmt::Stmt::Return(clause) => self.execute_return(clause),
            crate::query::parser::ast::stmt::Stmt::With(clause) => self.execute_with(clause),
            crate::query::parser::ast::stmt::Stmt::Yield(clause) => self.execute_yield(clause),
            crate::query::parser::ast::stmt::Stmt::Set(clause) => self.execute_set(clause),
            crate::query::parser::ast::stmt::Stmt::Remove(clause) => self.execute_remove(clause),
            crate::query::parser::ast::stmt::Stmt::Pipe(clause) => self.execute_pipe(clause),
            _ => Err(DBError::Query(QueryError::ExecutionError(
                "不支持的语句类型".to_string(),
            ))),
        }
    }
}
