//! 执行器工厂模块
//!
//! 负责根据执行计划创建对应的执行器实例
//! 采用简洁的工厂模式设计，职责单一，易于扩展

use crate::core::error::QueryError;
use crate::query::executor::traits::Executor;
use crate::query::planner::plan::core::{PlanNode, PlanNodeKind};
use crate::storage::StorageEngine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// 执行器创建器特征
///
/// 定义创建执行器的统一接口，支持对象安全的设计
pub trait ExecutorCreator<S: StorageEngine>: std::fmt::Debug + Send + Sync {
    /// 根据计划节点创建执行器实例
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError>;
}

/// 执行器工厂
///
/// 负责管理执行器创建器的注册和分发，职责单一
#[derive(Debug)]
pub struct ExecutorFactory<S: StorageEngine + 'static> {
    /// 执行器创建器映射表
    creators: HashMap<PlanNodeKind, Box<dyn ExecutorCreator<S>>>,
}

impl<S: StorageEngine + 'static + std::fmt::Debug> ExecutorFactory<S> {
    /// 创建新的执行器工厂
    pub fn new() -> Self {
        let mut factory = Self {
            creators: HashMap::new(),
        };

        // 注册默认的执行器创建器
        factory.register_default_creators();
        factory
    }

    /// 注册默认的执行器创建器
    fn register_default_creators(&mut self) {
        use crate::query::executor::factory::creators::*;

        // 数据访问执行器
        self.register_creator(
            PlanNodeKind::ScanVertices,
            Box::new(ScanVerticesCreator::new()),
        );
        self.register_creator(PlanNodeKind::ScanEdges, Box::new(ScanEdgesCreator::new()));

        // 结果处理执行器
        self.register_creator(PlanNodeKind::Filter, Box::new(FilterCreator::new()));
        self.register_creator(PlanNodeKind::Project, Box::new(ProjectCreator::new()));
        self.register_creator(PlanNodeKind::Limit, Box::new(LimitCreator::new()));
        self.register_creator(PlanNodeKind::Sort, Box::new(SortCreator::new()));
        self.register_creator(PlanNodeKind::Aggregate, Box::new(AggregateCreator::new()));

        // 数据处理执行器
        self.register_creator(PlanNodeKind::HashInnerJoin, Box::new(JoinCreator::new()));
        self.register_creator(PlanNodeKind::HashLeftJoin, Box::new(JoinCreator::new()));
        self.register_creator(PlanNodeKind::CartesianProduct, Box::new(JoinCreator::new()));

        // 图遍历执行器
        self.register_creator(PlanNodeKind::Expand, Box::new(ExpandCreator::new()));

        // 基础执行器
        self.register_creator(PlanNodeKind::Start, Box::new(StartCreator::new()));
        self.register_creator(PlanNodeKind::Unknown, Box::new(DefaultCreator::new()));
    }

    /// 注册执行器创建器
    pub fn register_creator(&mut self, kind: PlanNodeKind, creator: Box<dyn ExecutorCreator<S>>) {
        self.creators.insert(kind, creator);
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
        storage: Arc<Mutex<S>>,
    ) -> Result<Box<dyn Executor<S>>, QueryError> {
        let kind = plan_node.kind();

        let creator = self.creators.get(&kind).ok_or_else(|| {
            QueryError::ExecutionError(format!("未找到类型 {:?} 的执行器创建器", kind))
        })?;

        creator.create_executor(plan_node, storage)
    }
}

impl<S: StorageEngine + 'static + std::fmt::Debug> Default for ExecutorFactory<S> {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行器ID生成器
///
/// 专门负责生成唯一的执行器ID，职责单一
#[derive(Debug)]
pub struct ExecutorIdGenerator {
    next_id: usize,
}

impl ExecutorIdGenerator {
    /// 创建新的ID生成器
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    /// 生成下一个执行器ID
    pub fn generate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 获取下一个ID（不递增）
    pub fn next_id(&self) -> usize {
        self.next_id
    }
}

impl Default for ExecutorIdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// 执行器创建器模块
///
/// 包含所有具体的执行器创建器实现
pub mod creators {
    use super::*;
    use std::marker::PhantomData;

    // 数据访问执行器创建器
    mod data_access;
    pub use data_access::{ScanEdgesCreator, ScanVerticesCreator};

    // 结果处理执行器创建器
    mod result_processing;
    pub use result_processing::{
        AggregateCreator, FilterCreator, LimitCreator, ProjectCreator, SortCreator,
    };

    // 数据处理执行器创建器
    mod data_processing;
    pub use data_processing::JoinCreator;

    // 图遍历执行器创建器
    mod graph_traversal;
    pub use graph_traversal::ExpandCreator;

    // 基础执行器创建器
    mod base;
    pub use base::{DefaultCreator, StartCreator};
}

/// 聚合表达式解析工具
///
/// 提供聚合表达式的解析和验证功能
pub mod aggregation {
    use crate::core::error::QueryError;
    use crate::core::types::operators::AggregateFunction;
    use crate::query::executor::result_processing::aggregation::AggregateFunctionSpec;

    /// 解析聚合表达式字符串为AggregateFunctionSpec
    pub fn parse_aggregate_expression(expr_str: &str) -> Result<AggregateFunctionSpec, QueryError> {
        // 去除空白字符并转换为大写
        let expr = expr_str.trim().to_uppercase();

        // 检查表达式是否为空
        if expr.is_empty() {
            return Err(QueryError::ExecutionError("聚合表达式不能为空".to_string()));
        }

        // 解析常见的聚合函数模式
        if expr.starts_with("COUNT(") && expr.ends_with(")") {
            let content = &expr[6..expr.len() - 1].trim();
            if content == "*" || content == "1" {
                // COUNT(*) 或 COUNT(1)
                return Ok(AggregateFunctionSpec::count());
            } else {
                // COUNT(field)
                return Ok(AggregateFunctionSpec::count_distinct(content.to_string()));
            }
        }

        if expr.starts_with("SUM(") && expr.ends_with(")") {
            let field = expr[4..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::sum(field));
        }

        if expr.starts_with("AVG(") && expr.ends_with(")") {
            let field = expr[4..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::avg(field));
        }

        if expr.starts_with("MAX(") && expr.ends_with(")") {
            let field = expr[4..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::max(field));
        }

        if expr.starts_with("MIN(") && expr.ends_with(")") {
            let field = expr[4..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::min(field));
        }

        // 处理DISTINCT关键字
        if expr.starts_with("COUNT(DISTINCT") && expr.ends_with(")") {
            let field = expr[14..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::count_distinct(field));
        }

        // 处理其他支持的聚合函数
        if expr.starts_with("COLLECT(") && expr.ends_with(")") {
            let field = expr[8..expr.len() - 1].trim().to_string();
            return Ok(AggregateFunctionSpec::new(AggregateFunction::Collect).with_field(field));
        }

        // 如果无法识别聚合函数，返回错误而不是使用默认值
        Err(QueryError::ExecutionError(format!(
            "无法识别的聚合表达式: '{}'",
            expr_str
        )))
    }

    /// 验证聚合节点的参数
    pub fn validate_aggregate_node(
        group_keys: &[String],
        agg_exprs: &[String],
    ) -> Result<(), QueryError> {
        // 检查聚合表达式是否为空
        if agg_exprs.is_empty() {
            return Err(QueryError::ExecutionError(
                "聚合操作需要至少一个聚合表达式".to_string(),
            ));
        }

        // 验证分组键的有效性
        for key in group_keys {
            if key.trim().is_empty() {
                return Err(QueryError::ExecutionError(
                    "分组键不能为空字符串".to_string(),
                ));
            }
        }

        // 验证聚合表达式的有效性
        for expr in agg_exprs {
            if expr.trim().is_empty() {
                return Err(QueryError::ExecutionError(
                    "聚合表达式不能为空字符串".to_string(),
                ));
            }
        }

        Ok(())
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
        let factory = ExecutorFactory::<MockStorage>::new();
        assert!(!factory.creators.is_empty());
    }

    #[test]
    fn test_id_generator() {
        let mut generator = ExecutorIdGenerator::new();
        assert_eq!(generator.next_id(), 1);
        assert_eq!(generator.generate_id(), 1);
        assert_eq!(generator.next_id(), 2);
        assert_eq!(generator.generate_id(), 2);
    }

    #[test]
    fn test_aggregate_expression_parsing() {
        // 测试COUNT(*)
        let result = aggregation::parse_aggregate_expression("COUNT(*)");
        assert!(result.is_ok());

        // 测试COUNT(field)
        let result = aggregation::parse_aggregate_expression("COUNT(name)");
        assert!(result.is_ok());

        // 测试SUM(field)
        let result = aggregation::parse_aggregate_expression("SUM(age)");
        assert!(result.is_ok());

        // 测试无效表达式
        let result = aggregation::parse_aggregate_expression("INVALID()");
        assert!(result.is_err());
    }

    #[test]
    fn test_aggregate_validation() {
        // 测试有效参数
        let result = aggregation::validate_aggregate_node(
            &["category".to_string()],
            &["COUNT(*)".to_string()],
        );
        assert!(result.is_ok());

        // 测试空聚合表达式
        let result = aggregation::validate_aggregate_node(&["category".to_string()], &[]);
        assert!(result.is_err());

        // 测试空分组键
        let result =
            aggregation::validate_aggregate_node(&["".to_string()], &["COUNT(*)".to_string()]);
        assert!(result.is_err());
    }
}
