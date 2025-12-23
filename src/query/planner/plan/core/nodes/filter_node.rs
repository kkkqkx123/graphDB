//! 过滤节点实现
//!
//! FilterNode 用于根据指定的条件过滤输入数据流

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_kind::PlanNodeKind;
use crate::core::Expression;
use crate::query::context::validate::types::Variable;

/// 过滤节点
///
/// 根据指定的条件表达式过滤输入数据流
#[derive(Debug, Clone)]
pub struct FilterNode {
    id: i64,
    input: PlanNodeEnum,
    condition: Expression,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: PlanNodeEnum,
        condition: Expression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1, // 将在后续分配
            input,
            condition,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取过滤条件
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取节点的类型
    pub fn kind(&self) -> PlanNodeKind {
        PlanNodeKind::Filter
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> Vec<PlanNodeEnum> {
        vec![self.input.clone()]
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    /// 使用访问者模式访问节点
    pub fn accept(&self, visitor: &mut dyn crate::query::planner::plan::core::visitor::PlanNodeVisitor) -> Result<(), crate::query::planner::plan::core::visitor::PlanNodeVisitError> {
        visitor.visit_filter(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_filter_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node_enum = crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(start_node);

        let condition = Expression::Variable("test".to_string());
        let filter_node = FilterNode::new(start_node_enum, condition).expect("Filter node should be created successfully");

        assert_eq!(filter_node.kind(), PlanNodeKind::Filter);
        assert_eq!(filter_node.dependencies().len(), 1);
    }
}
