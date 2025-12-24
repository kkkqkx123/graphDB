//! 管理节点特征定义
//!
//! 定义所有管理节点需要实现的基础特征

use super::management_node_enum::ManagementNodeEnum;
use super::plan_node_enum::PlanNodeEnum;
use crate::core::error::PlanNodeVisitError;

/// 管理节点基础特征
///
/// 管理节点用于执行数据库管理操作，如创建/删除用户、角色、空间等
/// 与查询节点不同，管理节点不需要输出变量、列名等概念
pub trait ManagementNode {
    /// 获取节点的唯一ID
    fn id(&self) -> i64;

    /// 获取节点类型的名称
    fn name(&self) -> &'static str;

    /// 获取节点的成本估计值
    fn cost(&self) -> f64;

    /// 转换为 ManagementNodeEnum
    fn into_enum(self) -> ManagementNodeEnum;

    /// 获取依赖节点列表（管理节点默认无依赖）
    fn dependencies(&self) -> Vec<Box<PlanNodeEnum>> {
        Vec::new()
    }
}

/// 管理节点可克隆特征
pub trait ManagementNodeClonable {
    /// 克隆节点
    fn clone_management_node(&self) -> ManagementNodeEnum;

    /// 克隆节点并分配新的ID
    fn clone_with_new_id(&self, new_id: i64) -> ManagementNodeEnum;
}

/// 管理节点可访问特征
pub trait ManagementNodeVisitable {
    /// 接受访问者
    fn accept(&self, visitor: &mut dyn ManagementNodeVisitor) -> Result<(), PlanNodeVisitError>;
}

/// 管理节点访问者特征
pub trait ManagementNodeVisitor {
    /// 访问前处理
    fn pre_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }

    /// 访问后处理
    fn post_visit(&mut self) -> Result<(), PlanNodeVisitError> {
        Ok(())
    }
}
