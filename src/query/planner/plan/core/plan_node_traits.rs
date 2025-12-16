//! PlanNode trait 重构 - 拆分为多个小 trait
//!
//! 这个模块将原来的 PlanNode trait 拆分为多个职责单一的小 trait，
//! 遵循接口隔离原则，提高代码的可维护性和可扩展性。

use super::plan_node_kind::PlanNodeKind;
use super::visitor::{PlanNodeVisitError, PlanNodeVisitor};
use crate::query::context::validate::types::Variable;
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
    fn output_var(&self) -> &Option<Variable>;

    /// 获取列名列表
    fn col_names(&self) -> &Vec<String>;

    /// 获取节点的成本估计值
    fn cost(&self) -> f64;
}

/// 依赖管理 trait - 管理节点的依赖关系
pub trait PlanNodeDependencies {
    /// 获取节点的依赖节点列表
    fn dependencies(&self) -> &[Arc<dyn PlanNode>];

    /// 获取可变的依赖节点列表
    fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>>;

    /// 添加依赖节点
    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>);

    /// 移除指定ID的依赖节点
    fn remove_dependency(&mut self, id: i64) -> bool;
}

/// 可变性 trait - 提供节点的可变操作
pub trait PlanNodeMutable {
    /// 设置节点的输出变量
    fn set_output_var(&mut self, var: Variable);

    /// 设置列名
    fn set_col_names(&mut self, names: Vec<String>);

    /// 设置成本
    fn set_cost(&mut self, cost: f64);
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
}

/// 组合 trait - 组合所有 PlanNode 相关 trait
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
{
    /// 将节点作为Any类型返回，以支持downcast
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 为现有实现提供自动派生的宏
#[macro_export]
macro_rules! impl_plan_node_for {
    ($type:ty) => {
        impl PlanNodeIdentifiable for $type {
            fn id(&self) -> i64 {
                self.id
            }
            fn kind(&self) -> PlanNodeKind {
                self.kind.clone()
            }
        }

        impl PlanNodeProperties for $type {
            fn output_var(&self) -> &Option<Variable> {
                &self.output_var
            }
            fn col_names(&self) -> &Vec<String> {
                &self.col_names
            }
            fn cost(&self) -> f64 {
                self.cost
            }
        }

        impl PlanNodeDependencies for $type {
            fn dependencies(&self) -> &[Arc<dyn PlanNode>] {
                &self.dependencies
            }
            fn dependencies_mut(&mut self) -> &mut Vec<Arc<dyn PlanNode>> {
                &mut self.dependencies
            }

            fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
                self.dependencies.push(dep);
            }

            fn remove_dependency(&mut self, id: i64) -> bool {
                if let Some(pos) = self.dependencies.iter().position(|dep| dep.id() == id) {
                    self.dependencies.remove(pos);
                    true
                } else {
                    false
                }
            }
        }

        impl PlanNodeMutable for $type {
            fn set_output_var(&mut self, var: Variable) {
                self.output_var = Some(var);
            }

            fn set_col_names(&mut self, names: Vec<String>) {
                self.col_names = names;
            }

            fn set_cost(&mut self, cost: f64) {
                self.cost = cost;
            }
        }

        impl PlanNodeClonable for $type {
            fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
                Arc::new(self.clone())
            }
        }

        impl PlanNode for $type {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

/// 基础 PlanNode 结构体 - 用于所有权优化
#[derive(Debug, Clone)]
pub struct BasePlanNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub dependencies: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl BasePlanNode {
    /// 创建新的基础节点
    pub fn new(kind: PlanNodeKind) -> Self {
        Self {
            id: -1,
            kind,
            dependencies: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }

    /// 添加依赖节点（构建器模式）
    pub fn with_dependency(mut self, dep: Arc<dyn PlanNode>) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// 设置ID（构建器模式）
    pub fn with_id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// 设置输出变量（构建器模式）
    pub fn with_output_var(mut self, var: Variable) -> Self {
        self.output_var = Some(var);
        self
    }

    /// 设置列名（构建器模式）
    pub fn with_col_names(mut self, names: Vec<String>) -> Self {
        self.col_names = names;
        self
    }

    /// 设置成本（构建器模式）
    pub fn with_cost(mut self, cost: f64) -> Self {
        self.cost = cost;
        self
    }
}

// 为 BasePlanNode 实现所有 trait
impl_plan_node_for!(BasePlanNode);

impl PlanNodeVisitable for BasePlanNode {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }
}
