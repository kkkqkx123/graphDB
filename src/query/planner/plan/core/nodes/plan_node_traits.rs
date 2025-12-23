//! PlanNode 统一特征定义
//!
//! 定义所有计划节点需要实现的基础特征

use crate::query::context::validate::types::Variable;

/// PlanNode 基础特征
pub trait PlanNode {
    /// 获取节点的唯一ID
    fn id(&self) -> i64;

    /// 获取节点类型的名称
    fn name(&self) -> &'static str;

    /// 获取节点的输出变量
    fn output_var(&self) -> Option<&Variable>;

    /// 获取列名列表
    fn col_names(&self) -> &[String];

    /// 获取节点的成本估计值
    fn cost(&self) -> f64;

    /// 获取节点的依赖节点列表
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>>;

    /// 设置节点的输出变量
    fn set_output_var(&mut self, var: Variable);

    /// 设置列名
    fn set_col_names(&mut self, names: Vec<String>);

    /// 转换为 PlanNodeEnum
    fn into_enum(self) -> PlanNodeEnum;

    // 便捷方法访问特定输入

    /// 获取单个输入（适用于单输入节点）
    fn input(&self) -> Option<&PlanNodeEnum> {
        let deps = self.dependencies();
        if deps.len() == 1 {
            Some(&deps[0])
        } else {
            None
        }
    }

    /// 获取左输入（适用于双输入节点）
    fn left_input(&self) -> Option<&PlanNodeEnum> {
        let deps = self.dependencies();
        if deps.len() >= 1 {
            Some(&deps[0])
        } else {
            None
        }
    }

    /// 获取右输入（适用于双输入节点）
    fn right_input(&self) -> Option<&PlanNodeEnum> {
        let deps = self.dependencies();
        if deps.len() >= 2 {
            Some(&deps[1])
        } else {
            None
        }
    }

    /// 获取输入数量
    fn input_count(&self) -> usize {
        self.dependencies().len()
    }

    /// 获取指定索引的输入
    fn get_input(&self, index: usize) -> Option<&PlanNodeEnum> {
        self.dependencies().get(index).map(|boxed| boxed.as_ref())
    }
}

/// PlanNode 可克隆特征
pub trait PlanNodeClonable {
    /// 克隆节点
    fn clone_plan_node(&self) -> PlanNodeEnum;

    /// 克隆节点并分配新的ID
    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum;
}

/// PlanNode 可访问特征
pub trait PlanNodeVisitable {
    /// 接受访问者
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;
}

/// PlanNode 访问者特征
pub trait PlanNodeVisitor {
    /// 访问前处理
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问后处理
    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}

/// PlanNode 访问错误
#[derive(Debug, Clone)]
pub enum PlanNodeVisitError {
    /// 访问失败
    VisitFailed(String),
    /// 节点类型不匹配
    NodeTypeMismatch(String),
}

impl std::fmt::Display for PlanNodeVisitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanNodeVisitError::VisitFailed(msg) => write!(f, "访问失败: {}", msg),
            PlanNodeVisitError::NodeTypeMismatch(msg) => write!(f, "节点类型不匹配: {}", msg),
        }
    }
}

impl std::error::Error for PlanNodeVisitError {}

// 前向声明
use super::plan_node_enum::PlanNodeEnum;
