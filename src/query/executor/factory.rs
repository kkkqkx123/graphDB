//! 执行器工厂
//!
//! 负责根据执行计划创建对应的执行器实例
//! 基于nebula-graph的工厂模式设计

use crate::query::executor::traits::{Executor, ExecutorMetadata};
use crate::query::planner::plan::{PlanNode, PlanNodeKind};
use crate::query::types::{QueryError, QueryResult};
use crate::storage::StorageEngine;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::any::Any;

/// 执行器创建器特征 - 对象安全的设计
pub trait ExecutorCreator: std::fmt::Debug + Send + Sync {
    /// 创建执行器实例 - 返回Any以支持多种类型
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError>;
}

/// 基础执行器工厂
///
/// 负责根据计划节点类型创建对应的执行器
#[derive(Debug)]
pub struct ExecutorFactory {
    /// 执行器创建器映射
    creators: HashMap<PlanNodeKind, Box<dyn ExecutorCreator>>,
    /// 执行器ID计数器
    next_id: usize,
}

impl ExecutorFactory {
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
        self.register_creator(PlanNodeKind::ScanVertices, Box::new(ScanVerticesCreator));
        self.register_creator(PlanNodeKind::ScanEdges, Box::new(ScanEdgesCreator));
        self.register_creator(PlanNodeKind::Filter, Box::new(FilterCreator));
        self.register_creator(PlanNodeKind::Project, Box::new(ProjectCreator));
        self.register_creator(PlanNodeKind::Limit, Box::new(LimitCreator));
        self.register_creator(PlanNodeKind::Sort, Box::new(SortCreator));
        self.register_creator(PlanNodeKind::Aggregate, Box::new(AggregateCreator));
        self.register_creator(PlanNodeKind::Join, Box::new(JoinCreator));
        self.register_creator(PlanNodeKind::Expand, Box::new(ExpandCreator));
        self.register_creator(PlanNodeKind::Unknown, Box::new(DefaultCreator));
    }

    /// 注册执行器创建器
    pub fn register_creator(
        &mut self,
        kind: PlanNodeKind,
        creator: Box<dyn ExecutorCreator>,
    ) {
        self.creators.insert(kind, creator);
    }

    /// 根据计划节点创建执行器
    pub fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        let kind = plan_node.kind();
        
        let creator = self.creators.get(&kind).ok_or_else(|| {
            QueryError::ExecutionError(format!("未找到类型 {:?} 的执行器创建器", kind))
        })?;

        creator.create_executor(plan_node)
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

impl Default for ExecutorFactory {
    fn default() -> Self {
        Self::new()
    }
}

// 各种执行器创建器的实现

#[derive(Debug)]
struct ScanVerticesCreator;

impl ExecutorCreator for ScanVerticesCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现ScanVertices执行器
        Err(QueryError::ExecutionError("ScanVertices执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct ScanEdgesCreator;

impl ExecutorCreator for ScanEdgesCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现ScanEdges执行器
        Err(QueryError::ExecutionError("ScanEdges执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct FilterCreator;

impl ExecutorCreator for FilterCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Filter执行器
        Err(QueryError::ExecutionError("Filter执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct ProjectCreator;

impl ExecutorCreator for ProjectCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Project执行器
        Err(QueryError::ExecutionError("Project执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct LimitCreator;

impl ExecutorCreator for LimitCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Limit执行器
        Err(QueryError::ExecutionError("Limit执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct SortCreator;

impl ExecutorCreator for SortCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Sort执行器
        Err(QueryError::ExecutionError("Sort执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct AggregateCreator;

impl ExecutorCreator for AggregateCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Aggregate执行器
        Err(QueryError::ExecutionError("Aggregate执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct JoinCreator;

impl ExecutorCreator for JoinCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Join执行器
        Err(QueryError::ExecutionError("Join执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct ExpandCreator;

impl ExecutorCreator for ExpandCreator {
    fn create_executor(
        &self,
        _plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        // TODO: 实现Expand执行器
        Err(QueryError::ExecutionError("Expand执行器尚未实现".to_string()))
    }
}

#[derive(Debug)]
struct DefaultCreator;

impl ExecutorCreator for DefaultCreator {
    fn create_executor(
        &self,
        plan_node: &dyn PlanNode,
    ) -> Result<Box<dyn Any + Send + Sync>, QueryError> {
        Err(QueryError::ExecutionError(format!(
            "未知类型的计划节点: {:?}",
            plan_node.kind()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_creation() {
        let factory = ExecutorFactory::new();
        assert_eq!(factory.next_id(), 1);
    }

    #[test]
    fn test_generate_id() {
        let mut factory = ExecutorFactory::new();
        assert_eq!(factory.generate_id(), 1);
        assert_eq!(factory.generate_id(), 2);
        assert_eq!(factory.next_id(), 3);
    }
}