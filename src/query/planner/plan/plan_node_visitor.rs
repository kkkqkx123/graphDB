//! PlanNode访问者模式的定义
//! 用于遍历和处理计划树

use std::fmt;

/// 计划节点访问者特征
/// 用于实现访问者模式，遍历和处理计划树
pub trait PlanNodeVisitor: std::fmt::Debug {
    /// 在访问节点前调用的方法
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
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
