//! 过滤节点实现
//!
//! FilterNode 用于根据指定的条件过滤输入数据流

use crate::define_plan_node_with_deps;
use crate::core::Expression;
use super::plan_node_enum::PlanNodeEnum;

define_plan_node_with_deps! {
    pub struct FilterNode {
        condition: Expression,
    }
    enum: Filter
    input: SingleInputNode
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: PlanNodeEnum,
        condition: Expression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
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

    /// 设置过滤条件
    pub fn set_condition(&mut self, condition: Expression) {
        self.condition = condition;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_filter_node_creation() {
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node_enum =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                start_node,
            );

        let condition = Expression::Variable("test".to_string());
        let filter_node = FilterNode::new(start_node_enum, condition)
            .expect("Filter node should be created successfully");

        assert_eq!(filter_node.type_name(), "FilterNode");
        assert!(filter_node.condition().is_variable());
    }
}
