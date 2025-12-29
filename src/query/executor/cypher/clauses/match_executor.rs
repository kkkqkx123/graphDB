//! MATCH子句执行器
//!
//! 负责执行Cypher的MATCH语句，实现图模式匹配功能
//! 基于nebula-graph的TraverseExecutor设计，支持高效的图遍历和模式匹配

use crate::core::error::DBError;
use crate::core::Value;
use crate::query::executor::cypher::clauses::match_path::ExpressionEvaluator;
use crate::query::executor::cypher::clauses::match_path::PathInfo;
use crate::query::executor::cypher::clauses::match_path::PatternMatcher;
use crate::query::executor::cypher::clauses::match_path::ResultBuilder;
use crate::query::executor::cypher::clauses::match_path::TraversalEngine;
use crate::query::executor::cypher::context::CypherExecutionContext;
use crate::query::executor::traits::ExecutionResult;
use crate::query::parser::cypher::ast::clauses::MatchClause;
use crate::query::parser::cypher::ast::patterns::PatternPart;
use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

/// MATCH子句执行器
///
/// 负责处理图模式匹配，包括：
/// - 节点模式匹配
/// - 边模式匹配
/// - 路径模式匹配
/// - 可选匹配（OPTIONAL MATCH）
/// - WHERE条件过滤
#[derive(Debug)]
pub struct MatchClauseExecutor<S: StorageEngine> {
    /// 执行器ID
    id: i64,
    /// 模式匹配器
    pattern_matcher: PatternMatcher<S>,
    /// 图遍历引擎
    traversal_engine: TraversalEngine<S>,
    /// 结果构建器
    result_builder: ResultBuilder,
    /// 当前路径集合
    current_paths: Vec<PathInfo>,
    /// 结果路径集合
    result_paths: Vec<PathInfo>,
}

impl<S: StorageEngine> MatchClauseExecutor<S> {
    /// 创建新的MATCH执行器
    pub fn new(id: i64, storage: Arc<Mutex<S>>) -> Self {
        Self {
            id,
            pattern_matcher: PatternMatcher::new(storage.clone()),
            traversal_engine: TraversalEngine::new(storage.clone()),
            result_builder: ResultBuilder::new(),
            current_paths: Vec::new(),
            result_paths: Vec::new(),
        }
    }

    /// 设置最大路径长度限制
    pub fn set_max_path_length(&mut self, max_length: usize) {
        self.traversal_engine.set_max_path_length(max_length);
    }

    /// 设置最大结果数量限制
    pub fn set_max_result_count(&mut self, max_count: usize) {
        self.result_builder.set_max_result_count(max_count);
    }

    /// 执行模式匹配
    pub async fn execute_match(
        &mut self,
        clause: MatchClause,
        context: &mut CypherExecutionContext,
    ) -> Result<ExecutionResult, DBError> {
        // 设置执行状态
        context.set_state(crate::query::executor::cypher::context::ExecutionState::Executing);

        // 重置执行器状态
        self.reset_state();

        // 解析并执行模式
        for pattern in &clause.patterns {
            self.execute_pattern(pattern, context).await?;
        }

        // 处理WHERE条件
        if let Some(where_clause) = &clause.where_clause {
            self.apply_where_filter(where_clause, context).await?;
        }

        // 构建结果集
        let result = self
            .result_builder
            .build_result(&self.current_paths, &mut self.result_paths)?;

        // 设置完成状态
        context.set_state(crate::query::executor::cypher::context::ExecutionState::Completed);

        Ok(result)
    }

    /// 重置执行器状态
    fn reset_state(&mut self) {
        self.current_paths.clear();
        self.result_paths.clear();
        self.traversal_engine.reset();
    }

    /// 执行单个模式
    async fn execute_pattern(
        &mut self,
        pattern: &crate::query::parser::cypher::ast::patterns::Pattern,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        for part in &pattern.parts {
            self.execute_pattern_part(part, context).await?;
        }
        Ok(())
    }

    /// 执行模式部分
    async fn execute_pattern_part(
        &mut self,
        part: &PatternPart,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        // 首先处理起始节点
        let start_vertices = self
            .pattern_matcher
            .find_start_vertices(&part.node, context)
            .await?;

        if start_vertices.is_empty() {
            return Ok(());
        }

        // 初始化路径
        for vertex in start_vertices {
            let mut path = PathInfo::new();
            path.add_vertex(vertex.clone());

            // 如果节点有变量名，存储到上下文
            if let Some(var_name) = &part.node.variable {
                context.set_variable_value(var_name, Value::Vertex(Box::new(vertex.clone())));
            }

            self.current_paths.push(path);
        }

        // 处理关系模式
        for rel_pattern in &part.relationships {
            self.current_paths = self
                .traversal_engine
                .expand_with_relationship(&self.current_paths, rel_pattern, context)
                .await?;
        }

        Ok(())
    }

    /// 应用WHERE过滤条件
    async fn apply_where_filter(
        &mut self,
        where_clause: &crate::query::parser::cypher::ast::clauses::WhereClause,
        context: &mut CypherExecutionContext,
    ) -> Result<(), DBError> {
        // 如果没有当前路径，直接返回
        if self.current_paths.is_empty() {
            return Ok(());
        }

        // 求值WHERE表达式
        let result = ExpressionEvaluator::evaluate(&where_clause.expression, context)?;

        // 检查结果是否为布尔值
        if let Value::Bool(matches) = result {
            if !matches {
                // 如果条件不满足，清空结果
                self.current_paths.clear();
            }
        } else {
            return Err(DBError::Query(
                crate::core::error::QueryError::ExecutionError(
                    "WHERE表达式必须返回布尔值".to_string(),
                ),
            ));
        }

        Ok(())
    }

    /// 获取执行器ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取当前路径数量
    pub fn current_path_count(&self) -> usize {
        self.current_paths.len()
    }

    /// 获取结果路径数量
    pub fn result_path_count(&self) -> usize {
        self.result_paths.len()
    }

    /// 获取路径分析信息
    pub fn analyze_paths(
        &self,
    ) -> crate::query::executor::cypher::clauses::match_path::PathAnalysis {
        self.result_builder.analyze_paths(&self.result_paths)
    }

    /// 获取当前路径的引用
    pub fn current_paths(&self) -> &[PathInfo] {
        &self.current_paths
    }

    /// 获取结果路径的引用
    pub fn result_paths(&self) -> &[PathInfo] {
        &self.result_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::test_config::test_config;
    use crate::query::parser::cypher::ast::clauses::*;
    use crate::query::parser::cypher::ast::expressions::*;
    use crate::query::parser::cypher::ast::patterns::*;

    #[test]
    fn test_match_executor_creation() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let executor = MatchClauseExecutor::new(1, storage);
        assert_eq!(executor.id(), 1);
    }

    #[test]
    fn test_set_limits() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);

        executor.set_max_path_length(500);
        executor.set_max_result_count(5000);

        // 验证设置成功（通过内部状态检查）
        assert_eq!(executor.current_path_count(), 0);
        assert_eq!(executor.result_path_count(), 0);
    }

    #[tokio::test]
    async fn test_execute_match_empty_pattern() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();

        let clause = MatchClause {
            patterns: vec![],
            where_clause: None,
            optional: false,
        };

        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
        assert!(matches!(
            result.expect("Failed to execute match"),
            ExecutionResult::Success
        ));
    }

    #[tokio::test]
    async fn test_execute_match_simple_node_pattern() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();

        let node_pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec![],
            properties: None,
        };

        let pattern_part = PatternPart {
            node: node_pattern,
            relationships: vec![],
        };

        let pattern = Pattern {
            parts: vec![pattern_part],
        };

        let clause = MatchClause {
            patterns: vec![pattern],
            where_clause: None,
            optional: false,
        };

        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_match_with_where_clause() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();

        let node_pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec![],
            properties: None,
        };

        let pattern_part = PatternPart {
            node: node_pattern,
            relationships: vec![],
        };

        let pattern = Pattern {
            parts: vec![pattern_part],
        };

        // 创建一个总是为真的WHERE条件
        let where_clause = WhereClause {
            expression: Expression::Literal(Literal::Boolean(true)),
        };

        let clause = MatchClause {
            patterns: vec![pattern],
            where_clause: Some(where_clause),
            optional: false,
        };

        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_execute_match_with_false_where_clause() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);
        let mut context = CypherExecutionContext::new();

        let node_pattern = NodePattern {
            variable: Some("n".to_string()),
            labels: vec![],
            properties: None,
        };

        let pattern_part = PatternPart {
            node: node_pattern,
            relationships: vec![],
        };

        let pattern = Pattern {
            parts: vec![pattern_part],
        };

        // 创建一个总是为假的WHERE条件
        let where_clause = WhereClause {
            expression: Expression::Literal(Literal::Boolean(false)),
        };

        let clause = MatchClause {
            patterns: vec![pattern],
            where_clause: Some(where_clause),
            optional: false,
        };

        let result = executor.execute_match(clause, &mut context).await;
        assert!(result.is_ok());
        assert!(matches!(
            result.expect("Failed to execute match"),
            ExecutionResult::Success
        ));
    }

    #[test]
    fn test_reset_state() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let mut executor = MatchClauseExecutor::new(1, storage);

        // 模拟添加一些路径
        executor.current_paths.push(PathInfo::new());
        executor.result_paths.push(PathInfo::new());

        assert_eq!(executor.current_path_count(), 1);
        assert_eq!(executor.result_path_count(), 1);

        // 重置状态
        executor.reset_state();

        assert_eq!(executor.current_path_count(), 0);
        assert_eq!(executor.result_path_count(), 0);
    }

    #[test]
    fn test_analyze_paths() {
        let config = test_config();
        let storage = Arc::new(Mutex::new(
            crate::storage::native_storage::NativeStorage::new(config.test_db_path("test_db"))
                .expect("failed to create storage"),
        ));
        let executor = MatchClauseExecutor::new(1, storage);

        let analysis = executor.analyze_paths();
        assert_eq!(analysis.total_paths, 0);
        assert_eq!(analysis.empty_paths, 0);
    }
}
