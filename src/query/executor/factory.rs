//! 执行器工厂模块
//!
//! 负责根据执行计划创建对应的执行器实例
//! 采用直接匹配模式，简单高效，易于维护

use crate::core::error::QueryError;
use crate::query::executor::traits::Executor;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{PlanNode, JoinNode, BinaryInputNode};

use crate::storage::StorageEngine;
use std::sync::{Arc, Mutex};

// 导入已实现的执行器
use crate::query::executor::data_access::GetVerticesExecutor;
use crate::query::executor::result_processing::{
    FilterExecutor, ProjectExecutor, LimitExecutor, SortExecutor, AggregateExecutor, DedupExecutor
};
use crate::query::executor::data_processing::{
    ExpandExecutor, InnerJoinExecutor, LeftJoinExecutor, CrossJoinExecutor,
    UnwindExecutor, AssignExecutor
};
use crate::query::executor::base::{StartExecutor, ExecutionContext};
use crate::query::executor::recursion_detector::{RecursionDetector, ExecutorSafetyValidator, ExecutorSafetyConfig};

/// 执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器实例
/// 采用直接匹配模式，避免过度抽象
/// 包含递归检测和安全验证机制
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    storage: Option<Arc<Mutex<S>>>,
    recursion_detector: RecursionDetector,
    safety_validator: ExecutorSafetyValidator,
}

impl<S: StorageEngine + 'static> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let recursion_detector = RecursionDetector::new(100);
        let safety_validator = ExecutorSafetyValidator::new(ExecutorSafetyConfig::default());
        
        Self {
            storage: None,
            recursion_detector,
            safety_validator,
        }
    }
    
    /// 设置存储引擎
    pub fn with_storage(storage: Arc<Mutex<S>>) -> Self {
        let recursion_detector = RecursionDetector::new(100);
        let safety_validator = ExecutorSafetyValidator::new(ExecutorSafetyConfig::default());
        
        Self {
            storage: Some(storage),
            recursion_detector,
            safety_validator,
        }
    }

    /// 提取连接操作的变量名
    fn extract_join_vars<N: JoinNode>(node: &N) -> (String, String) {
        let left_var = node.left_input().output_var()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| format!("left_{}", node.id()));
        let right_var = node.right_input().output_var()
            .map(|v| v.name.clone())
            .unwrap_or_else(|| format!("right_{}", node.id()));
        (left_var, right_var)
    }

    /// 创建内连接执行器（通用方法）
    fn create_inner_join_executor<N>(
        &self,
        node: &N,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError>
    where
        N: JoinNode,
    {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let executor = InnerJoinExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            node.hash_keys().to_vec(),
            node.probe_keys().to_vec(),
            node.col_names().to_vec(),
        );
        Ok(Box::new(executor))
    }

    /// 创建左连接执行器（通用方法）
    fn create_left_join_executor<N>(
        &self,
        node: &N,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError>
    where
        N: JoinNode,
    {
        let (left_var, right_var) = Self::extract_join_vars(node);
        let executor = LeftJoinExecutor::new(
            node.id(),
            storage,
            left_var,
            right_var,
            node.hash_keys().to_vec(),
            node.probe_keys().to_vec(),
            node.col_names().to_vec(),
        );
        Ok(Box::new(executor))
    }

    /// 验证计划节点的安全性
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        match plan_node {
            PlanNodeEnum::Expand(node) => {
                let step_limit = node.step_limit().and_then(|s| usize::try_from(s).ok()).unwrap_or(10);
                if step_limit > 1000 {
                    return Err(QueryError::ExecutionError(
                        format!("Expand执行器的步数限制{}超过安全阈值1000", step_limit)
                    ));
                }
            }
            PlanNodeEnum::Loop(_) => {
                return Err(QueryError::ExecutionError(
                    "循环执行器需要手动构建，不支持通过工厂自动创建".to_string()
                ));
            }
            _ => {}
        }
        Ok(())
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &self,
        plan_node: &PlanNodeEnum,
        storage: Arc<Mutex<S>>,
        context: &ExecutionContext,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        // 验证执行器类型和配置
        self.validate_plan_node(plan_node)?;
        
        match plan_node {
            // 基础执行器
            PlanNodeEnum::Start(node) => {
                Ok(Box::new(StartExecutor::new(node.id())))
            }

            // 数据访问执行器
            PlanNodeEnum::ScanVertices(node) => {
                let executor = GetVerticesExecutor::new(
                    node.id(),
                    storage,
                    None,
                    node.tag_filter().as_ref().and_then(|f| crate::query::parser::expressions::parse_expression_from_string(f).ok()),
                    node.vertex_filter().as_ref().and_then(|f| crate::query::parser::expressions::parse_expression_from_string(f).ok()),
                    node.limit().map(|l| l as usize),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::ScanEdges(_) => {
                // TODO: 需要实现扫描边执行器
                Err(QueryError::ExecutionError(
                    "扫描边执行器尚未实现".to_string(),
                ))
            }
            PlanNodeEnum::GetVertices(node) => {
                let executor = GetVerticesExecutor::new(
                    node.id(),
                    storage,
                    Some(vec![crate::core::Value::String(node.src_vids().to_string())]),
                    None,
                    node.expr().and_then(|e| crate::query::parser::expressions::parse_expression_from_string(e).ok()),
                    node.limit().map(|l| l as usize),
                );
                Ok(Box::new(executor))
            }

            // 结果处理执行器
            PlanNodeEnum::Filter(node) => {
                let executor = FilterExecutor::new(
                    node.id(),
                    storage,
                    node.condition().clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Project(node) => {
                let columns = node.columns().iter().map(|col| {
                    crate::query::executor::result_processing::ProjectionColumn::new(
                        col.alias.clone(),
                        col.expr.clone(),
                    )
                }).collect();
                let executor = ProjectExecutor::new(
                    node.id(),
                    storage,
                    columns,
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Limit(node) => {
                let executor = LimitExecutor::new(
                    node.id(),
                    storage,
                    Some(node.count() as usize),
                    node.offset() as usize,
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Sort(node) => {
                let sort_keys = node.sort_items().iter().map(|item| {
                    crate::query::executor::result_processing::SortKey::new(
                        crate::core::Expression::Variable(item.clone()),
                        crate::query::executor::result_processing::SortOrder::Asc,
                    )
                }).collect();
                let config = crate::query::executor::result_processing::SortConfig::default();
                let executor = SortExecutor::new(
                    node.id(),
                    storage,
                    sort_keys,
                    node.limit().map(|l| l as usize),
                    config,
                ).map_err(|e| QueryError::ExecutionError(e.to_string()))?;
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Aggregate(node) => {
                let aggregate_functions = node.agg_exprs().iter().map(|expr| {
                    crate::query::executor::result_processing::AggregateFunctionSpec::new(
                        crate::core::types::operators::AggregateFunction::Count,
                    )
                }).collect();
                let group_by_expressions = node.group_keys().iter().map(|key| {
                    crate::core::Expression::Variable(key.clone())
                }).collect();
                let executor = AggregateExecutor::new(
                    node.id(),
                    storage,
                    aggregate_functions,
                    group_by_expressions,
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Dedup(node) => {
                let executor = DedupExecutor::new(
                    node.id(),
                    storage,
                    crate::query::executor::result_processing::DedupStrategy::Full,
                    None,
                );
                Ok(Box::new(executor))
            }

            // 数据处理执行器
            PlanNodeEnum::InnerJoin(node) => {
                self.create_inner_join_executor(&node, storage)
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                self.create_inner_join_executor(&node, storage)
            }
            PlanNodeEnum::LeftJoin(node) => {
                self.create_left_join_executor(&node, storage)
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                self.create_left_join_executor(&node, storage)
            }
            PlanNodeEnum::CrossJoin(node) | PlanNodeEnum::CartesianProduct(node) => {
                let left_var = node.left_input().output_var()
                    .map(|v| v.name.to_string())
                    .unwrap_or_else(|| format!("left_{}", node.id()));
                let right_var = node.right_input().output_var()
                    .map(|v| v.name.to_string())
                    .unwrap_or_else(|| format!("right_{}", node.id()));
                let executor = CrossJoinExecutor::new(
                    node.id(),
                    storage,
                    vec![left_var, right_var],
                    node.col_names().to_vec(),
                );
                Ok(Box::new(executor))
            }

            // 图遍历执行器
            PlanNodeEnum::Expand(node) => {
                // 验证Expand执行器的安全配置
                self.safety_validator.validate_expand_config(
                    node.step_limit().and_then(|s| usize::try_from(s).ok()),
                ).map_err(|e| QueryError::ExecutionError(e.to_string()))?;
                
                let executor = ExpandExecutor::new(
                    node.id(),
                    storage,
                    node.direction(),
                    if node.edge_types().is_empty() {
                        None
                    } else {
                        Some(node.edge_types().to_vec())
                    },
                    node.step_limit().and_then(|s| usize::try_from(s).ok()),
                );
                Ok(Box::new(executor))
            }
            
            // 数据转换执行器
            PlanNodeEnum::Unwind(node) => {
                let unwind_expr = crate::query::parser::expressions::parse_expression_from_string(node.list_expr())
                    .map_err(|e| QueryError::ExecutionError(format!("解析表达式失败: {}", e)))?;
                let executor = UnwindExecutor::new(
                    node.id(),
                    storage,
                    node.alias().to_string(),
                    unwind_expr,
                    node.col_names().to_vec(),
                    false,
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Assign(node) => {
                let mut parsed_assignments = Vec::new();
                for (var_name, expr_str) in node.assignments() {
                    let expr = crate::query::parser::expressions::parse_expression_from_string(expr_str)
                        .map_err(|e| QueryError::ExecutionError(format!("解析表达式失败: {}", e)))?;
                    parsed_assignments.push((var_name.clone(), expr));
                }
                let executor = AssignExecutor::new(
                    node.id(),
                    storage,
                    parsed_assignments,
                );
                Ok(Box::new(executor))
            }
            
            // 循环执行器
            PlanNodeEnum::Loop(node) => {
                // 注意：循环执行器需要body_executor，这里暂时返回错误
                // 在实际使用中，需要在构建循环执行器时传入body_executor
                Err(QueryError::ExecutionError(
                    "循环执行器需要body_executor，请在构建时传入".to_string()
                ))
            }

            _ => Err(QueryError::ExecutionError(format!(
                "暂不支持执行器类型: {:?}",
                plan_node.type_name()
            ))),
        }
    }

    /// 执行执行计划
    pub async fn execute_plan(
        &self,
        _query_context: &mut crate::core::context::query::QueryContext,
        plan: crate::query::planner::plan::ExecutionPlan,
    ) -> Result<crate::query::executor::traits::ExecutionResult, QueryError> {
        // 获取存储引擎
        let storage = match &self.storage {
            Some(storage) => storage.clone(),
            None => return Err(QueryError::ExecutionError("存储引擎未设置".to_string())),
        };
        
        // 创建执行上下文
        let execution_context = ExecutionContext::new();
        
        // 设置会话和数据库信息到执行上下文中
        // 注意：ExecutionContext 结构可能需要扩展以支持这些字段
        // 目前我们使用基本的执行上下文，后续可以根据需要扩展
        
        // 获取根节点
        let root_node = match plan.root() {
            Some(node) => node,
            None => return Err(QueryError::ExecutionError("执行计划没有根节点".to_string())),
        };
        
        // 创建根执行器
        let mut executor = self.create_executor(
            root_node,
            storage,
            &execution_context,
        )?;
        
        // 执行根执行器
        let result = executor.execute().await.map_err(|e| {
            QueryError::ExecutionError(format!("执行器执行失败: {}", e))
        })?;
        
        // 返回执行结果
        Ok(result)
    }
}

impl<S: StorageEngine + 'static> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::test_mock::MockStorage;

    #[test]
    fn test_factory_creation() {
        let _factory = ExecutorFactory::<MockStorage>::new();
        // 工厂创建成功
    }

    #[test]
    fn test_create_unsupported_executor() {
        let factory = ExecutorFactory::<MockStorage>::new();
        let storage = Arc::new(Mutex::new(MockStorage));
        let plan_node =
            PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::StartNode::new());
        let context = ExecutionContext::new();

        let result = factory.create_executor(&plan_node, storage, &context);
        match result {
            Err(e) => assert!(e.to_string().contains("尚未实现")),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }
}
