//! PlanNode 统一特征定义
//!
//! 定义所有计划节点需要实现的基础特征

use crate::core::error::PlanNodeVisitError;
use crate::core::Expression;
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

    /// 设置节点的输出变量
    fn set_output_var(&mut self, var: Variable);

    /// 设置列名
    fn set_col_names(&mut self, names: Vec<String>);

    /// 转换为 PlanNodeEnum
    fn into_enum(self) -> PlanNodeEnum;
}

/// 为引用类型实现 PlanNode trait
impl<T: PlanNode + ?Sized> PlanNode for &T {
    fn id(&self) -> i64 {
        (**self).id()
    }

    fn name(&self) -> &'static str {
        (**self).name()
    }

    fn output_var(&self) -> Option<&Variable> {
        (**self).output_var()
    }

    fn col_names(&self) -> &[String] {
        (**self).col_names()
    }

    fn cost(&self) -> f64 {
        (**self).cost()
    }

    fn set_output_var(&mut self, _var: Variable) {
        panic!("无法通过引用修改输出变量")
    }

    fn set_col_names(&mut self, _names: Vec<String>) {
        panic!("无法通过引用修改列名")
    }

    fn into_enum(self) -> PlanNodeEnum {
        panic!("无法将引用转换为 PlanNodeEnum")
    }
}

/// 单输入节点特征
///
/// 适用于只有一个输入的节点
pub trait SingleInputNode: PlanNode {
    /// 获取输入节点
    fn input(&self) -> &PlanNodeEnum;

    /// 设置输入节点
    fn set_input(&mut self, input: PlanNodeEnum);

    /// 获取输入数量（始终为1）
    fn input_count(&self) -> usize {
        1
    }
}

/// 双输入节点特征
///
/// 适用于有两个输入的节点（如连接操作）
pub trait BinaryInputNode: PlanNode {
    /// 获取左输入节点
    fn left_input(&self) -> &PlanNodeEnum;

    /// 获取右输入节点
    fn right_input(&self) -> &PlanNodeEnum;

    /// 设置左输入节点
    fn set_left_input(&mut self, input: PlanNodeEnum);

    /// 设置右输入节点
    fn set_right_input(&mut self, input: PlanNodeEnum);

    /// 获取输入数量（始终为2）
    fn input_count(&self) -> usize {
        2
    }
}

/// 为引用类型实现 BinaryInputNode trait
impl<T: BinaryInputNode + ?Sized> BinaryInputNode for &T {
    fn left_input(&self) -> &PlanNodeEnum {
        (**self).left_input()
    }

    fn right_input(&self) -> &PlanNodeEnum {
        (**self).right_input()
    }

    fn set_left_input(&mut self, _input: PlanNodeEnum) {
        panic!("无法通过引用修改输入节点")
    }

    fn set_right_input(&mut self, _input: PlanNodeEnum) {
        panic!("无法通过引用修改输入节点")
    }
}

/// 连接节点特征
///
/// 适用于所有类型的连接操作（内连接、左连接、交叉连接等）
/// 统一了连接节点的接口，便于在执行器工厂中统一处理
pub trait JoinNode: BinaryInputNode {
    /// 获取哈希键（用于构建哈希表）
    fn hash_keys(&self) -> &[Expression];

    /// 获取探测键（用于探测哈希表）
    fn probe_keys(&self) -> &[Expression];
}

/// 为引用类型实现 JoinNode trait
impl<T: JoinNode + ?Sized> JoinNode for &T {
    fn hash_keys(&self) -> &[Expression] {
        (**self).hash_keys()
    }

    fn probe_keys(&self) -> &[Expression] {
        (**self).probe_keys()
    }
}

/// 多输入节点特征
///
/// 适用于有多个输入的节点（如Union）
pub trait MultipleInputNode: PlanNode {
    /// 获取所有输入节点
    fn inputs(&self) -> &[Box<PlanNodeEnum>];

    /// 添加输入节点
    fn add_input(&mut self, input: PlanNodeEnum);

    /// 移除指定索引的输入节点
    fn remove_input(&mut self, index: usize) -> Result<(), String>;

    /// 获取输入数量
    fn input_count(&self) -> usize {
        self.inputs().len()
    }
}

/// 无输入节点特征
///
/// 适用于没有输入的节点（如Start）
pub trait ZeroInputNode: PlanNode {
    /// 获取输入数量（始终为0）
    fn input_count(&self) -> usize {
        0
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

// 前向声明
use super::plan_node_enum::PlanNodeEnum;
