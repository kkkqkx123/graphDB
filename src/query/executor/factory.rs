//! 执行器工厂
//!
//! 负责根据执行计划创建对应的执行器实例
//! 基于nebula-graph的工厂模式设计

use crate::graph::expression::Expression;
use crate::query::executor::traits::Executor;
use crate::query::planner::plan::{PlanNode, PlanNodeKind};
use crate::query::types::QueryError;
use crate::storage::StorageEngine;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// 执行器创建器特征 - 对象安全的设计
pub trait ExecutorCreator<S: StorageEngine>: std::fmt::Debug + Send + Sync {
    /// 创建执行器实例 - 返回具体的执行器类型
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError>;
}

/// 基础执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器
#[derive(Debug)]
pub struct BaseExecutorFactory<S: StorageEngine + 'static> {
    /// 执行器创建器映射
    creators: HashMap<PlanNodeKind, Box<dyn ExecutorCreator<S>>>,
    /// 执行器ID计数器
    next_id: usize,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> BaseExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
            next_id: 1,
        };

        // 注册默认的执行器创建器
        factory.register_default_creators();
        factory
    }

    /// 注册默认的执行器创建器
    fn register_default_creators(&mut self) {
        // 注册各种计划节点类型的执行器创建器
        self.register_creator(
            PlanNodeKind::ScanVertices,
            Box::new(ScanVerticesCreator::<S> { _phantom: PhantomData }),
        );
        self.register_creator(PlanNodeKind::ScanEdges, Box::new(ScanEdgesCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Filter, Box::new(FilterCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Project, Box::new(ProjectCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Limit, Box::new(LimitCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Sort, Box::new(SortCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Aggregate, Box::new(AggregateCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::InnerJoin, Box::new(JoinCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Expand, Box::new(ExpandCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Start, Box::new(StartCreator::<S> { _phantom: PhantomData }));
        self.register_creator(PlanNodeKind::Unknown, Box::new(DefaultCreator::<S> { _phantom: PhantomData }));
    }

    /// 注册执行器创建器
    pub fn register_creator(&mut self, kind: PlanNodeKind, creator: Box<dyn ExecutorCreator<S>>) {
        self.creators.insert(kind, creator);
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &mut self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let kind = plan_node.kind();

        let creator = self.creators.get(&kind).ok_or_else(|| {
            QueryError::ExecutionError(format!("未找到类型 {:?} 的执行器创建器", kind))
        })?;

        creator.create_executor(plan_node, storage)
    }

    /// 获取下一个执行器ID
    pub fn next_id(&self) -> usize {
        self.next_id
    }

    /// 生成并获取下一个执行器ID
    pub fn generate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

impl<S: StorageEngine + 'static + std::fmt::Debug> Default for BaseExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

// 各种执行器创建器的实现

#[derive(Debug)]
struct ScanVerticesCreator<S: StorageEngine + std::fmt::Debug> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ScanVerticesCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_access::GetVerticesExecutor;
        use crate::query::planner::plan::ScanVertices;

        let id = plan_node.id() as usize;

        // 尝试从具体的ScanVertices计划节点中提取参数
        let (vertex_ids, tag_filter) =
            if let Some(_scan_node) = plan_node.as_any().downcast_ref::<ScanVertices>() {
                // TODO: 这里需要解析顶点ID和标签过滤条件
                // 暂时使用None
                (None, None)
            } else {
                // 如果不是具体的ScanVertices节点，使用默认值
                (None, None)
            };

        let executor = GetVerticesExecutor::new(id, storage, vertex_ids, tag_filter);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct ScanEdgesCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ScanEdgesCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_access::GetEdgesExecutor;
        use crate::query::planner::plan::ScanEdges;

        let id = plan_node.id() as usize;

        // 尝试从具体的ScanEdges计划节点中提取参数
        let edge_filter = if let Some(_scan_node) = plan_node.as_any().downcast_ref::<ScanEdges>() {
            // TODO: 这里需要解析边过滤条件
            // 暂时使用None
            None
        } else {
            // 如果不是具体的ScanEdges节点，使用默认值
            None
        };

        let executor = GetEdgesExecutor::new(id, storage, edge_filter);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct FilterCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for FilterCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::graph::expression::Expression;
        use crate::query::executor::data_processing::filter::FilterExecutor;
        use crate::query::planner::plan::operations::data_processing_ops::Filter;

        let id = plan_node.id() as usize;

        // 尝试从具体的Filter计划节点中提取条件
        let condition = if let Some(_filter_node) = plan_node.as_any().downcast_ref::<Filter>() {
            // 解析过滤条件字符串为表达式
            // TODO: 这里需要实现表达式解析器
            // 暂时使用简单的true表达式
            Expression::literal(true)
        } else {
            // 如果不是具体的Filter节点，使用默认条件
            Expression::literal(true)
        };

        let executor = FilterExecutor::new(id, storage, condition);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct ProjectCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ProjectCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::result_processing::projection::ProjectExecutor;
        use crate::query::executor::result_processing::projection::ProjectionColumn;
        use crate::query::planner::plan::operations::data_processing_ops::Project;

        let id = plan_node.id() as usize;

        // 尝试从具体的Project计划节点中提取投影表达式
        let columns = if let Some(project_node) = plan_node.as_any().downcast_ref::<Project>() {
            // 解析投影表达式字符串
            // TODO: 这里需要实现表达式解析器
            // 暂时根据yield_expr创建简单的投影列
            if project_node.yield_expr == "*" {
                vec![ProjectionColumn::new("*".to_string(), Expression::literal("*"))]
            } else {
                // 简单分割表达式，创建投影列
                project_node
                    .yield_expr
                    .split(',')
                    .map(|expr| ProjectionColumn::new(expr.trim().to_string(), Expression::variable(expr.trim().to_string())))
                    .collect()
            }
        } else {
            // 如果不是具体的Project节点，使用默认投影
            vec![ProjectionColumn::new("*".to_string(), Expression::literal("*"))]
        };

        let executor = ProjectExecutor::new(id, storage, columns);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct LimitCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for LimitCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::pagination::LimitExecutor;
        use crate::query::planner::plan::operations::sorting_ops::Limit;

        let id = plan_node.id() as usize;

        // 尝试从具体的Limit计划节点中提取参数
        let (limit, offset) = if let Some(limit_node) = plan_node.as_any().downcast_ref::<Limit>() {
            (limit_node.count.try_into().ok(), limit_node.offset.try_into().unwrap_or(0))
        } else {
            // 如果不是具体的Limit节点，使用默认值
            (None, 0)
        };

        let executor = LimitExecutor::new(id, storage, limit, offset);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct SortCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for SortCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::graph::expression::Expression;
        use crate::query::executor::data_processing::sort::{SortExecutor, SortKey, SortOrder};
        use crate::query::planner::plan::operations::sorting_ops::Sort;

        let id = plan_node.id() as usize;

        // 尝试从具体的Sort计划节点中提取参数
        let (sort_keys, limit) = if let Some(sort_node) = plan_node.as_any().downcast_ref::<Sort>()
        {
            // 解析排序字段
            let keys: Vec<SortKey> = sort_node
                .sort_items
                .iter()
                .map(|item| {
                    // TODO: 这里需要解析排序方向，暂时默认为升序
                    SortKey::new(Expression::variable(item.clone()), SortOrder::Asc)
                })
                .collect();

            (keys, sort_node.limit.and_then(|l| l.try_into().ok()))
        } else {
            // 如果不是具体的Sort节点，使用默认值
            let default_keys = vec![SortKey::new(
                Expression::variable("default".to_string()),
                SortOrder::Asc,
            )];
            (default_keys, None)
        };

        let executor = SortExecutor::new(id, storage, sort_keys, limit);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct AggregateCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for AggregateCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::aggregation::AggregateExecutor;
        use crate::query::planner::plan::operations::aggregation_ops::Aggregate;

        let id = plan_node.id() as usize;

        // 尝试从具体的Aggregate计划节点中提取参数
        let (group_keys, agg_funcs): (Vec<String>, Vec<String>) =
            if let Some(_agg_node) = plan_node.as_any().downcast_ref::<Aggregate>() {
                // TODO: 这里需要解析分组键和聚合函数
                // 暂时使用空列表
                (vec![], vec![])
            } else {
                // 如果不是具体的Aggregate节点，使用默认值
                (vec![], vec![])
            };

        let executor = AggregateExecutor::new(id, storage);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct JoinCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for JoinCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::data_processing::join::inner_join::InnerJoinExecutor;
        // use crate::query::planner::plan::operations::join_ops::Join;
        // TODO: 修复 Join 导入问题

        let id = plan_node.id() as usize;

        // 尝试从具体的Join计划节点中提取参数
        let (left_var, right_var, left_keys, right_keys, output_cols) =
            if let Some(_join_node) = plan_node.as_any().downcast_ref::<()>() {
                // TODO: 这里需要解析连接条件
                // 暂时使用默认值
                (
                    "left".to_string(),
                    "right".to_string(),
                    vec!["0".to_string()],
                    vec!["0".to_string()],
                    vec!["id".to_string(), "name".to_string()],
                )
            } else {
                // 如果不是具体的Join节点，使用默认值
                (
                    "left".to_string(),
                    "right".to_string(),
                    vec!["0".to_string()],
                    vec!["0".to_string()],
                    vec!["id".to_string(), "name".to_string()],
                )
            };

        let executor = InnerJoinExecutor::new(
            id,
            storage,
            left_var,
            right_var,
            left_keys,
            right_keys,
            output_cols,
        );
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct ExpandCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for ExpandCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::base::EdgeDirection;
        use crate::query::executor::data_processing::graph_traversal::expand::ExpandExecutor;
        use crate::query::planner::plan::operations::traversal_ops::Expand;

        let id = plan_node.id() as usize;

        // 尝试从具体的Expand计划节点中提取参数
        let (direction, edge_types, vertex_filter) =
            if let Some(_expand_node) = plan_node.as_any().downcast_ref::<Expand>() {
                // TODO: 这里需要解析展开参数
                // 暂时使用默认值
                (EdgeDirection::Both, None, None)
            } else {
                // 如果不是具体的Expand节点，使用默认值
                (EdgeDirection::Both, None, None)
            };

        let executor = ExpandExecutor::new(id, storage, direction, edge_types, vertex_filter);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct StartCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for StartCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        use crate::query::executor::base::StartExecutor;

        let id = plan_node.id() as usize;

        // 创建基础的Start执行器
        let executor = StartExecutor::new(id, storage);
        Ok(Box::new(executor))
    }
}

#[derive(Debug)]
struct DefaultCreator<S: StorageEngine + std::fmt::Debug + 'static> {
    _phantom: PhantomData<S>,
}

impl<S: StorageEngine + std::fmt::Debug + 'static> ExecutorCreator<S> for DefaultCreator<S> {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        _storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        Err(QueryError::ExecutionError(format!(
            "未知类型的计划节点: {:?}",
            plan_node.kind()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageEngine;

    // 模拟存储引擎用于测试
    struct MockStorage;

    impl StorageEngine for MockStorage {
        fn insert_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<crate::core::Value, crate::storage::StorageError> {
            unimplemented!()
        }

        fn get_node(
            &self,
            _id: &crate::core::Value,
        ) -> Result<Option<crate::core::vertex_edge_path::Vertex>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn update_node(
            &mut self,
            _vertex: crate::core::vertex_edge_path::Vertex,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn delete_node(
            &mut self,
            _id: &crate::core::Value,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn insert_edge(
            &mut self,
            _edge: crate::core::vertex_edge_path::Edge,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn get_edge(
            &self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<Option<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn get_node_edges(
            &self,
            _node_id: &crate::core::Value,
            _direction: crate::core::vertex_edge_path::Direction,
        ) -> Result<Vec<crate::core::vertex_edge_path::Edge>, crate::storage::StorageError>
        {
            unimplemented!()
        }

        fn delete_edge(
            &mut self,
            _src: &crate::core::Value,
            _dst: &crate::core::Value,
            _edge_type: &str,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn begin_transaction(&mut self) -> Result<u64, crate::storage::StorageError> {
            unimplemented!()
        }

        fn commit_transaction(&mut self, _tx_id: u64) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }

        fn rollback_transaction(
            &mut self,
            _tx_id: u64,
        ) -> Result<(), crate::storage::StorageError> {
            unimplemented!()
        }
    }

    #[test]
    fn test_factory_creation() {
        let factory = BaseExecutorFactory::<MockStorage>::new();
        assert_eq!(factory.next_id(), 1);
    }

    #[test]
    fn test_generate_id() {
        let mut factory = BaseExecutorFactory::<MockStorage>::new();
        assert_eq!(factory.generate_id(), 1);
        assert_eq!(factory.generate_id(), 2);
        assert_eq!(factory.next_id(), 3);
    }
}
