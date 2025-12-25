//! 执行器工厂模块
//!
//! 负责根据执行计划创建对应的执行器实例
//! 采用直接匹配模式，简单高效，易于维护

use crate::core::error::QueryError;
use crate::query::executor::traits::Executor;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

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

/// 执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器实例
/// 采用直接匹配模式，避免过度抽象
#[derive(Debug)]
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    storage: Option<Arc<Mutex<S>>>,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        Self {
            storage: None,
        }
    }
    
    /// 设置存储引擎
    pub fn with_storage(storage: Arc<Mutex<S>>) -> Self {
        Self {
            storage: Some(storage),
        }
    }

    /// 验证计划节点是否有效
    fn validate_plan_node(&self, plan_node: &PlanNodeEnum) -> Result<(), QueryError> {
        // 检查节点配置是否有效
        match plan_node {
            PlanNodeEnum::Limit(node) => {
                if node.limit == 0 {
                    return Err(QueryError::ExecutionError("LIMIT值不能为0".to_string()));
                }
            }
            PlanNodeEnum::Loop(node) => {
                if let Some(max_iter) = node.max_iterations {
                    if max_iter == 0 {
                        return Err(QueryError::ExecutionError("最大迭代次数不能为0".to_string()));
                    }
                    if max_iter > 10000 {
                        return Err(QueryError::ExecutionError("最大迭代次数不能超过10000".to_string()));
                    }
                }
            }
            PlanNodeEnum::Expand(node) => {
                if let Some(max_depth) = node.max_depth {
                    if max_depth == 0 {
                        return Err(QueryError::ExecutionError("扩展深度不能为0".to_string()));
                    }
                    if max_depth > 100 {
                        return Err(QueryError::ExecutionError("扩展深度不能超过100".to_string()));
                    }
                }
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
        // ✅ 添加执行计划验证
        self.validate_plan_node(plan_node)?;
        
        match plan_node {
            // 基础执行器
            PlanNodeEnum::Start(node) => {
                Ok(Box::new(StartExecutor::new(node.id, storage)))
            }

            // 数据访问执行器
            PlanNodeEnum::ScanVertices(node) => {
                // 创建扫描顶点执行器
                let executor = GetVerticesExecutor::new(
                    node.id,
                    storage,
                    None, // vertex_ids - 扫描所有顶点
                    node.tag_filter.clone(),
                    node.vertex_filter.clone(),
                    node.limit,
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
                // 创建获取顶点执行器
                let executor = GetVerticesExecutor::new(
                    node.id,
                    storage,
                    node.vertex_ids.clone(),
                    node.tag_filter.clone(),
                    node.vertex_filter.clone(),
                    node.limit,
                );
                Ok(Box::new(executor))
            }

            // 结果处理执行器
            PlanNodeEnum::Filter(node) => {
                let executor = FilterExecutor::new(
                    node.id,
                    storage,
                    node.filter_expression.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Project(node) => {
                let executor = ProjectExecutor::new(
                    node.id,
                    storage,
                    node.projections.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Limit(node) => {
                let executor = LimitExecutor::new(
                    node.id,
                    storage,
                    node.limit,
                    node.offset,
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Sort(node) => {
                let executor = SortExecutor::new(
                    node.id,
                    storage,
                    node.sort_keys.clone(),
                    node.sort_orders.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Aggregate(node) => {
                let executor = AggregateExecutor::new(
                    node.id,
                    storage,
                    node.aggregate_functions.clone(),
                    node.group_by_expressions.clone(),
                    node.having_expression.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Dedup(node) => {
                let executor = DedupExecutor::new(
                    node.id,
                    storage,
                    node.dedup_keys.clone(),
                    node.dedup_strategy.clone(),
                );
                Ok(Box::new(executor))
            }

            // 数据处理执行器
            PlanNodeEnum::InnerJoin(node) | PlanNodeEnum::HashInnerJoin(node) => {
                let executor = InnerJoinExecutor::new(
                    node.id,
                    storage,
                    node.join_condition.clone(),
                    node.join_type.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::LeftJoin(node) | PlanNodeEnum::HashLeftJoin(node) => {
                let executor = LeftJoinExecutor::new(
                    node.id,
                    storage,
                    node.join_condition.clone(),
                    node.join_type.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::CrossJoin(node) | PlanNodeEnum::CartesianProduct(node) => {
                let executor = CrossJoinExecutor::new(
                    node.id,
                    storage,
                    node.join_type.clone(),
                );
                Ok(Box::new(executor))
            }

            // 图遍历执行器
            PlanNodeEnum::Expand(node) => {
                let executor = ExpandExecutor::new(
                    node.id,
                    storage,
                    node.edge_direction.clone(),
                    node.edge_types.clone(),
                    node.max_depth,
                );
                Ok(Box::new(executor))
            }
            
            // 数据转换执行器
            PlanNodeEnum::Unwind(node) => {
                let executor = UnwindExecutor::new(
                    node.id,
                    storage,
                    node.unwind_expression.clone(),
                    node.unwind_variable.clone(),
                );
                Ok(Box::new(executor))
            }
            PlanNodeEnum::Assign(node) => {
                let executor = AssignExecutor::new(
                    node.id,
                    storage,
                    node.assignments.clone(),
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

impl<S: StorageEngine + 'static + std::fmt::Debug> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageEngine;

    // 模拟存储引擎用于测试
    #[derive(Debug)]
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            Ok(crate::core::Value::Null(crate::core::value::NullType::NaN))
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn scan_all_vertices(
            &self,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn scan_vertices_by_tag(
            &self,
            _tag: &str,
        ) -> Result<Vec<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(None)
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            Ok(Vec::new())
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            Ok(1)
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            Ok(())
        }
    }

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

        let result = factory.create_executor(&plan_node, storage);
        match result {
            Err(e) => assert!(e.to_string().contains("尚未实现")),
            Ok(_) => panic!("Expected error but got Ok"),
        }
    }
}
