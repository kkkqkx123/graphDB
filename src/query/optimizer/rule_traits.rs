//! 通用规则trait和工具函数
//! 提供优化规则的通用接口和辅助函数，减少代码重复

use super::optimizer::{OptContext, OptGroupNode, OptRule, Pattern, OptimizerError};
use crate::query::planner::plan::{PlanNode, PlanNodeKind};

/// 优化规则的基础trait，扩展了OptRule
pub trait BaseOptRule: OptRule {
    /// 获取规则的优先级，数值越小优先级越高
    fn priority(&self) -> u32 {
        100 // 默认优先级
    }
    
    /// 检查规则是否适用于给定的计划节点
    fn is_applicable(&self, node: &OptGroupNode) -> bool {
        self.pattern().matches(node)
    }
    
    /// 应用规则前的验证
    fn validate(&self, ctx: &OptContext, node: &OptGroupNode) -> Result<(), OptimizerError> {
        // 默认实现不做任何验证
        Ok(())
    }
    
    /// 应用规则后的处理
    fn post_process(&self, ctx: &mut OptContext, original_node: &OptGroupNode, result_node: &OptGroupNode) -> Result<(), OptimizerError> {
        // 默认实现不做任何处理
        Ok(())
    }
}

/// 下推优化规则的通用trait
pub trait PushDownRule: BaseOptRule {
    /// 检查是否可以下推到指定的子节点类型
    fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool;
    
    /// 获取下推后的新节点
    fn create_pushed_down_node(&self, ctx: &mut OptContext, node: &OptGroupNode, child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 合并优化规则的通用trait
pub trait MergeRule: BaseOptRule {
    /// 检查是否可以合并指定的节点
    fn can_merge(&self, node: &OptGroupNode, child: &OptGroupNode) -> bool;
    
    /// 创建合并后的新节点
    fn create_merged_node(&self, ctx: &mut OptContext, node: &OptGroupNode, child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 消除优化规则的通用trait
pub trait EliminationRule: BaseOptRule {
    /// 检查节点是否可以被消除
    fn can_eliminate(&self, node: &OptGroupNode) -> bool;
    
    /// 获取消除后的替代节点（如果有）
    fn get_replacement(&self, ctx: &mut OptContext, node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError>;
}

/// 辅助函数：检查条件是否为永真式
pub fn is_tautology(condition: &str) -> bool {
    match condition.trim() {
        "1 = 1" | "true" | "TRUE" | "True" => true,
        // 检查更复杂的永真式，如 a = a
        _ => {
            // 尝试解析表达式并检查是否为永真式
            // 这里应该使用完整的表达式解析器
            false
        }
    }
}

/// 辅助函数：合并两个过滤条件
pub fn combine_conditions(cond1: &str, cond2: &str) -> String {
    if cond1.is_empty() {
        cond2.to_string()
    } else if cond2.is_empty() {
        cond1.to_string()
    } else {
        format!("({}) AND ({})", cond1, cond2)
    }
}

/// 辅助函数：合并表达式列表
pub fn combine_expression_list(exprs: &[String]) -> String {
    if exprs.is_empty() {
        String::new()
    } else if exprs.len() == 1 {
        exprs[0].clone()
    } else {
        format!("({})", exprs.join(") AND ("))
    }
}

/// 辅助结构：表示过滤条件分离的结果
#[derive(Debug, Clone)]
pub struct FilterSplitResult {
    pub pushable_condition: Option<String>,  // 可以下推的条件
    pub remaining_condition: Option<String>, // 保留在Filter节点的条件
}

/// 辅助函数：创建基本的模式匹配
pub fn create_basic_pattern(kind: PlanNodeKind) -> Pattern {
    Pattern::new(kind)
}

/// 辅助函数：创建带依赖的模式匹配
pub fn create_pattern_with_dependency(kind: PlanNodeKind, dependency_kind: PlanNodeKind) -> Pattern {
    Pattern::new(kind).with_dependency(Pattern::new(dependency_kind))
}

/// 辅助函数：检查节点是否有指定类型的依赖
pub fn has_dependency_of_kind(node: &OptGroupNode, kind: PlanNodeKind) -> bool {
    // 这里需要实际的依赖检查逻辑
    // 当前简化实现
    false
}

/// 辅助函数：获取节点的第一个依赖
pub fn get_first_dependency(node: &OptGroupNode) -> Option<&OptGroupNode> {
    // 这里需要实际的依赖获取逻辑
    // 当前简化实现
    None
}

/// 辅助函数：创建新的OptGroupNode
pub fn create_new_opt_group_node(id: usize, plan_node: Box<dyn PlanNode>) -> OptGroupNode {
    OptGroupNode::new(id, plan_node)
}

/// 辅助函数：克隆OptGroupNode但替换计划节点
pub fn clone_with_new_plan_node(node: &OptGroupNode, plan_node: Box<dyn PlanNode>) -> OptGroupNode {
    let mut new_node = node.clone();
    new_node.plan_node = plan_node;
    new_node
}

/// 宏：简化规则实现的重复代码
#[macro_export]
macro_rules! impl_basic_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }
        
        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }
    };
}

/// 宏：简化下推规则的实现
#[macro_export]
macro_rules! impl_push_down_rule {
    ($rule_type:ty, $name:expr, $target_kind:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }
        
        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }
        
        impl PushDownRule for $rule_type {
            fn can_push_down_to(&self, child_kind: PlanNodeKind) -> bool {
                child_kind == $target_kind
            }
        }
    };
}

/// 宏：简化合并规则的实现
#[macro_export]
macro_rules! impl_merge_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }
        
        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }
        
        impl MergeRule for $rule_type {
            // 默认实现，需要具体规则重写
            fn can_merge(&self, _node: &OptGroupNode, _child: &OptGroupNode) -> bool {
                false
            }
            
            fn create_merged_node(&self, _ctx: &mut OptContext, _node: &OptGroupNode, _child: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
                Ok(None)
            }
        }
    };
}

/// 宏：简化消除规则的实现
#[macro_export]
macro_rules! impl_elimination_rule {
    ($rule_type:ty, $name:expr) => {
        impl OptRule for $rule_type {
            fn name(&self) -> &str {
                $name
            }
        }
        
        impl BaseOptRule for $rule_type {
            // 使用默认实现
        }
        
        impl EliminationRule for $rule_type {
            // 默认实现，需要具体规则重写
            fn can_eliminate(&self, _node: &OptGroupNode) -> bool {
                false
            }
            
            fn get_replacement(&self, _ctx: &mut OptContext, _node: &OptGroupNode) -> Result<Option<OptGroupNode>, OptimizerError> {
                Ok(None)
            }
        }
    };
}