//! 聚合节点实现
//!
//! AggregateNode 用于对输入数据进行聚合操作

use crate::define_plan_node_with_deps;
use crate::core::types::operators::AggregateFunction;

define_plan_node_with_deps! {
    pub struct AggregateNode {
        group_keys: Vec<String>,
        aggregation_functions: Vec<AggregateFunction>,
    }
    enum: Aggregate
    input: SingleInputNode
}

impl AggregateNode {
    /// 创建新的聚合节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        group_keys: Vec<String>,
        aggregation_functions: Vec<AggregateFunction>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names: Vec<String> = group_keys.clone();
        for agg_func in &aggregation_functions {
            col_names.push(agg_func.name().to_string());
        }

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            group_keys,
            aggregation_functions,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取分组键
    pub fn group_keys(&self) -> &[String] {
        &self.group_keys
    }

    /// 获取聚合函数列表
    pub fn aggregation_functions(&self) -> &[AggregateFunction] {
        &self.aggregation_functions
    }

    /// 获取聚合表达式（别名方法，与aggregation_functions相同）
    pub fn agg_exprs(&self) -> &[AggregateFunction] {
        &self.aggregation_functions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                crate::query::planner::plan::core::nodes::start_node::StartNode::new(),
            );

        let group_keys = vec!["category".to_string()];
        let aggregation_functions = vec![AggregateFunction::Count(None)];

        let aggregate_node = AggregateNode::new(start_node, group_keys, aggregation_functions)
            .expect("Aggregate node should be created successfully");

        assert_eq!(aggregate_node.type_name(), "AggregateNode");
        assert_eq!(aggregate_node.dependencies().len(), 1);
        assert_eq!(aggregate_node.group_keys().len(), 1);
        assert_eq!(aggregate_node.aggregation_functions().len(), 1);
    }
}
