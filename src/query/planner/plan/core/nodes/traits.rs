//! PlanNode trait 定义
//!
//! 这个模块定义了所有计划节点相关的 trait，遵循接口隔离原则。

use super::super::plan_node_kind::PlanNodeKind;
use super::super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::core::context::validate::types::Variable;
use std::sync::Arc;

/// 基础标识 trait - 提供节点的基本标识信息
pub trait PlanNodeIdentifiable {
    /// 获取节点的唯一ID
    fn id(&self) -> i64;

    /// 获取节点的类型
    fn kind(&self) -> PlanNodeKind;
}

/// 属性访问 trait - 提供节点的属性访问
pub trait PlanNodeProperties {
    /// 获取节点的输出变量
    fn output_var(&self) -> Option<&Variable>;

    /// 获取列名列表
    fn col_names(&self) -> &[String];

    /// 获取节点的成本估计值
    fn cost(&self) -> f64;
}

/// 依赖管理 trait - 管理节点的依赖关系
pub trait PlanNodeDependencies {
    /// 获取节点的依赖节点列表（返回克隆以避免生命周期问题）
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>>;

    /// 获取依赖节点的数量
    fn dependency_count(&self) -> usize {
        self.dependencies().len()
    }

    /// 添加依赖节点（主要用于构建阶段）
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>);

    /// 移除指定ID的依赖节点
    fn remove_dependency(&mut self, id: i64) -> bool;
}

/// 依赖管理扩展 trait - 提供更安全的访问方式
pub trait PlanNodeDependenciesExt {
    /// 使用闭包访问依赖节点列表（更安全的访问方式）
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R;

    /// 检查是否包含指定ID的依赖
    fn has_dependency(&self, id: i64) -> bool {
        self.with_dependencies(|deps| deps.iter().any(|dep| dep.id() == id))
    }
}

/// 可变性 trait - 提供节点的可变操作
pub trait PlanNodeMutable {
    /// 设置节点的输出变量
    fn set_output_var(&mut self, var: Variable);

    /// 设置列名
    fn set_col_names(&mut self, names: Vec<String>);
}

/// 访问者支持 trait - 支持访问者模式
pub trait PlanNodeVisitable {
    /// 使用访问者模式访问节点
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError>;
}

/// 克隆支持 trait - 提供节点的克隆功能
pub trait PlanNodeClonable {
    /// 克隆节点
    fn clone_plan_node(&self) -> Arc<dyn PlanNode>;

    /// 克隆节点并分配新的ID
    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode>;
}

/// 组合 trait - 组合所有 PlanNode 相关 trait
#[allow(clippy::type_complexity)]
pub trait PlanNode:
    PlanNodeIdentifiable
    + PlanNodeProperties
    + PlanNodeDependencies
    + PlanNodeMutable
    + PlanNodeVisitable
    + PlanNodeClonable
    + Send
    + Sync
    + std::fmt::Debug
    + 'static
{
    /// 将节点作为Any类型返回，以支持downcast
    fn as_any(&self) -> &dyn std::any::Any;
}
