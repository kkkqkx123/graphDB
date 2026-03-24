//! 集合操作节点实现
//!
//! 提供集合操作相关的计划节点定义

use crate::define_plan_node_with_deps;

define_plan_node_with_deps! {
    pub struct MinusNode {
    }
    enum: Minus
    input: SingleInputNode
}

impl MinusNode {
    pub fn new(
        input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
        minus_input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
    ) -> Result<Self, crate::query::planning::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input), Box::new(minus_input)],
            output_var: None,
            col_names,
        })
    }

    pub fn minus_input(
        &self,
    ) -> &crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        &self.deps[1]
    }
}

define_plan_node_with_deps! {
    pub struct IntersectNode {
    }
    enum: Intersect
    input: SingleInputNode
}

impl IntersectNode {
    pub fn new(
        input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
        intersect_input: crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum,
    ) -> Result<Self, crate::query::planning::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input), Box::new(intersect_input)],
            output_var: None,
            col_names,
        })
    }

    pub fn intersect_input(
        &self,
    ) -> &crate::query::planning::plan::core::nodes::base::plan_node_enum::PlanNodeEnum {
        &self.deps[1]
    }
}
