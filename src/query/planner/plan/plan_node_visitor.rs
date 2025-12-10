//! PlanNode访问者模式的定义
//! 用于遍历和处理计划树

use std::fmt;
use crate::query::planner::plan::nodes::{GetVertices, GetEdges, Project, Filter, Dedup, Expand, ExpandAll, Start, Argument, HashLeftJoin, HashInnerJoin};

/// 计划节点访问者特征
/// 用于实现访问者模式，遍历和处理计划树
pub trait PlanNodeVisitor: std::fmt::Debug {
    /// 在访问节点前调用的方法
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问GetVertices节点
    fn visit_get_vertices(&mut self, _node: &GetVertices) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问GetEdges节点
    fn visit_get_edges(&mut self, _node: &GetEdges) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Project节点
    fn visit_project(&mut self, _node: &Project) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Filter节点
    fn visit_filter(&mut self, _node: &Filter) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Dedup节点
    fn visit_dedup(&mut self, _node: &Dedup) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Expand节点
    fn visit_expand(&mut self, _node: &Expand) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问ExpandAll节点
    fn visit_expand_all(&mut self, _node: &ExpandAll) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Start节点
    fn visit_start(&mut self, _node: &Start) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问Argument节点
    fn visit_argument(&mut self, _node: &Argument) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问HashLeftJoin节点
    fn visit_hash_left_join(&mut self, _node: &HashLeftJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问HashInnerJoin节点
    fn visit_hash_inner_join(&mut self, _node: &HashInnerJoin) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 在访问节点后调用的方法
    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}

/// 计划节点访问错误
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    /// 访问错误
    VisitError(String),

    /// 遍历错误
    TraversalError(String),

    /// 验证错误
    ValidationError(String),
}

impl fmt::Display for PlanNodeVisitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanNodeVisitError::VisitError(msg) => write!(f, "访问错误: {}", msg),
            PlanNodeVisitError::TraversalError(msg) => write!(f, "遍历错误: {}", msg),
            PlanNodeVisitError::ValidationError(msg) => write!(f, "验证错误: {}", msg),
        }
    }
}

impl std::error::Error for PlanNodeVisitError {}

/// 具体的计划节点访问者实现示例
#[derive(Debug)]
pub struct DefaultPlanNodeVisitor;

impl PlanNodeVisitor for DefaultPlanNodeVisitor {
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}
