//! 聚合节点实现
//!
//! AggregateNode 用于对输入数据进行聚合操作

use crate::core::types::operators::AggregateFunction;
use crate::query::context::validate::types::Variable;

/// 聚合节点
///
/// 根据指定的分组键和聚合表达式对输入数据进行聚合
#[derive(Debug, Clone)]
pub struct AggregateNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    group_keys: Vec<String>,
    /// 聚合函数列表，替代原来的agg_exprs
    aggregation_functions: Vec<AggregateFunction>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl AggregateNode {
    /// 创建新的聚合节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        group_keys: Vec<String>,
        aggregation_functions: Vec<AggregateFunction>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        // 生成列名：分组键 + 聚合函数名称
        let mut col_names: Vec<String> = group_keys.clone();
        for agg_func in &aggregation_functions {
            col_names.push(agg_func.name().to_string());
        }

        let mut deps = Vec::new();
        deps.push(Box::new(input.clone()));

        Ok(Self {
            id: -1,
            input: Box::new(input),
            deps,
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

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Aggregate"
    }

    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn cost(&self) -> f64 {
        self.cost
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.deps
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(dep.clone());
        self.deps.clear();
        self.deps.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, _id: i64) -> bool {
        // 聚合节点只支持单个输入，这个方法在当前设计中不太适用
        false
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Aggregate(Self {
            id: self.id,
            input: self.input.clone(),
            deps: self.deps.clone(),
            group_keys: self.group_keys.clone(),
            aggregation_functions: self.aggregation_functions.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Aggregate(cloned)
    }
}

// 为 AggregateNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for AggregateNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn cost(&self) -> f64 {
        self.cost()
    }

    fn set_output_var(&mut self, var: Variable) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Aggregate(self)
    }
}

// 为 AggregateNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for AggregateNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

// 为 AggregateNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for AggregateNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::operators::AggregateFunction;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_aggregate_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let group_keys = vec!["category".to_string()];
        let aggregation_functions = vec![AggregateFunction::Count(None)];

        let aggregate_node = AggregateNode::new(start_node, group_keys, aggregation_functions)
            .expect("Aggregate node should be created successfully");

        assert_eq!(aggregate_node.type_name(), "Aggregate");
        assert_eq!(aggregate_node.dependencies().len(), 1);
        assert_eq!(aggregate_node.group_keys().len(), 1);
        assert_eq!(aggregate_node.aggregation_functions().len(), 1);
    }
}
