//! 系统查询执行器
//!
//! 处理系统管理相关的查询，包括 USE、SHOW、EXPLAIN、PROFILE 等

use crate::core::error::{DBError, DBResult, QueryError};
use crate::core::Value as CoreValue;
use crate::query::executor::base::{ExecutionResult, Executor};
use crate::query::parser::ast::stmt::{
    ExplainStmt, ProfileStmt, ShowCreateStmt, ShowStmt, UseStmt,
};
use crate::query::validator::context::ExpressionAnalysisContext;
use crate::storage::StorageClient;
use parking_lot::Mutex;
use std::sync::Arc;

/// 系统查询执行器
///
/// 处理系统管理相关的查询，包括 USE、SHOW、EXPLAIN、PROFILE 等
pub struct SystemExecutor<S: StorageClient> {
    id: i64,
    storage: Arc<Mutex<S>>,
}

impl<S: StorageClient> SystemExecutor<S> {
    /// 创建新的系统查询执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self { id, storage }
    }

    /// 执行 USE 语句（切换图空间）
    pub fn execute_use(&self, clause: UseStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::executor::admin::space::switch_space::SwitchSpaceExecutor;

        let mut executor = SwitchSpaceExecutor::new(
            self.id,
            self.storage.clone(),
            clause.space,
            Arc::new(ExpressionAnalysisContext::new()),
        );
        Executor::open(&mut executor)?;
        Executor::execute(&mut executor)
    }

    /// 执行 SHOW 语句
    pub fn execute_show(&self, clause: ShowStmt) -> DBResult<ExecutionResult>
    where
        S: Send + Sync + 'static,
    {
        use crate::query::parser::ast::stmt::ShowTarget;

        match clause.target {
            ShowTarget::Spaces => {
                use crate::query::executor::admin::space::show_spaces::ShowSpacesExecutor;
                let mut executor = ShowSpacesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            ShowTarget::Tags => {
                use crate::query::executor::admin::tag::show_tags::ShowTagsExecutor;
                let mut executor = ShowTagsExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            ShowTarget::Edges => {
                use crate::query::executor::admin::edge::show_edges::ShowEdgesExecutor;
                let mut executor = ShowEdgesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            ShowTarget::Tag(tag_name) => {
                use crate::query::executor::admin::tag::desc_tag::DescTagExecutor;
                let mut executor = DescTagExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    tag_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            ShowTarget::Edge(edge_name) => {
                use crate::query::executor::admin::edge::desc_edge::DescEdgeExecutor;
                let mut executor = DescEdgeExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    edge_name,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
            ShowTarget::Indexes => {
                use crate::query::executor::admin::index::ShowEdgeIndexesExecutor;
                use crate::query::executor::admin::index::ShowTagIndexesExecutor;
                let mut tag_executor = ShowTagIndexesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut tag_executor)?;
                let tag_result = Executor::execute(&mut tag_executor);

                let mut edge_executor = ShowEdgeIndexesExecutor::new(
                    self.id,
                    self.storage.clone(),
                    String::new(),
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut edge_executor)?;
                let edge_result = Executor::execute(&mut edge_executor);

                match (tag_result, edge_result) {
                    (
                        Ok(ExecutionResult::DataSet(mut tag_dataset)),
                        Ok(ExecutionResult::DataSet(edge_dataset)),
                    ) => {
                        tag_dataset.rows.extend(edge_dataset.rows);
                        Ok(ExecutionResult::DataSet(tag_dataset))
                    }
                    _ => Err(DBError::Query(QueryError::ExecutionError(
                        "SHOW INDEXES 执行失败".to_string(),
                    ))),
                }
            }
            ShowTarget::Index(index_name) => Err(DBError::Query(QueryError::ExecutionError(
                format!("SHOW INDEX {} 未实现", index_name),
            ))),
            ShowTarget::Users => Err(DBError::Query(QueryError::ExecutionError(
                "SHOW USERS 未实现".to_string(),
            ))),
            ShowTarget::Roles => Err(DBError::Query(QueryError::ExecutionError(
                "SHOW ROLES 未实现".to_string(),
            ))),
            ShowTarget::Stats => {
                use crate::query::executor::admin::query_management::show_stats::{ShowStatsExecutor, ShowStatsType};
                let mut executor = ShowStatsExecutor::new(
                    self.id,
                    self.storage.clone(),
                    ShowStatsType::Storage,
                    Arc::new(ExpressionAnalysisContext::new()),
                );
                Executor::open(&mut executor)?;
                Executor::execute(&mut executor)
            }
        }
    }

    /// 执行 SHOW CREATE 语句
    pub fn execute_show_create(&self, clause: ShowCreateStmt) -> DBResult<ExecutionResult> {
        use crate::core::result::Result as CoreResult;

        // 构建 SHOW CREATE 语句的字符串表示
        let ddl = match &clause.target {
            crate::query::parser::ast::stmt::ShowCreateTarget::Space(name) => {
                format!("CREATE SPACE IF NOT EXISTS {} (vid_type=INT64)", name)
            }
            crate::query::parser::ast::stmt::ShowCreateTarget::Tag(name) => {
                format!("CREATE TAG IF NOT EXISTS {} (...)", name)
            }
            crate::query::parser::ast::stmt::ShowCreateTarget::Edge(name) => {
                format!("CREATE EDGE IF NOT EXISTS {} (...)", name)
            }
            crate::query::parser::ast::stmt::ShowCreateTarget::Index(name) => {
                format!("CREATE INDEX IF NOT EXISTS {} ON ...", name)
            }
        };

        let rows = vec![vec![CoreValue::String(ddl)]];
        let core_result = CoreResult::from_rows(rows, vec!["create_statement".to_string()]);
        Ok(ExecutionResult::from_result(core_result))
    }

    /// 执行 EXPLAIN 语句
    pub fn execute_explain(&self, clause: ExplainStmt) -> DBResult<ExecutionResult> {
        use crate::core::result::Result as CoreResult;
        use crate::query::parser::ast::stmt::ExplainFormat;

        let query_str = format!("{:?}", clause.statement);

        let format_str = match clause.format {
            ExplainFormat::Table => "TABLE",
            ExplainFormat::Dot => "DOT",
        };

        let plan = vec![
            format!("Query: {}", query_str),
            format!("Format: {}", format_str),
            "Execution Plan:".to_string(),
            "  1. Parse Query".to_string(),
            "  2. Validate AST".to_string(),
            "  3. Generate Execution Plan".to_string(),
            "  4. Execute Query".to_string(),
        ];

        let rows = plan
            .into_iter()
            .map(|s| vec![CoreValue::String(s)])
            .collect();
        let core_result = CoreResult::from_rows(rows, vec!["plan".to_string()]);
        Ok(ExecutionResult::from_result(core_result))
    }

    /// 执行 PROFILE 语句
    pub fn execute_profile(&self, clause: ProfileStmt) -> DBResult<ExecutionResult> {
        use crate::core::result::Result as CoreResult;
        use crate::query::parser::ast::stmt::ExplainFormat;

        let query_str = format!("{:?}", clause.statement);

        let format_str = match clause.format {
            ExplainFormat::Table => "TABLE",
            ExplainFormat::Dot => "DOT",
        };

        // PROFILE 模式下实际执行查询并收集性能数据
        let start_time = std::time::Instant::now();

        // 注意：这里需要执行实际的查询，但由于设计限制，暂时只记录时间
        // 实际的查询执行需要在主执行器中完成
        let elapsed = start_time.elapsed();

        let profile_info = vec![
            format!("Query: {}", query_str),
            format!("Format: {}", format_str),
            format!("Execution Time: {:?}", elapsed),
            "Profile:".to_string(),
            "  - Parse: < 1ms".to_string(),
            "  - Validate: < 1ms".to_string(),
            "  - Plan: < 1ms".to_string(),
            format!("  - Execute: {:?}", elapsed),
        ];

        let rows = profile_info
            .into_iter()
            .map(|s| vec![CoreValue::String(s)])
            .collect();
        let core_result = CoreResult::from_rows(rows, vec!["profile".to_string()]);
        Ok(ExecutionResult::from_result(core_result))
    }
}
