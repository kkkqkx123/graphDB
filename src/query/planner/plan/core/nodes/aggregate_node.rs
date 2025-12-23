//! 聚合节点实现
//!
//! AggregateNode 用于对输入数据进行聚合操作

use crate::query::context::validate::types::Variable;

/// 聚合节点
///
/// 根据指定的分组键和聚合表达式对输入数据进行聚合
#[derive(Debug, Clone)]
pub struct AggregateNode {
    id: i64,
    input: super::plan_node_enum::PlanNodeEnum,
    deps: Vec<super::plan_node_enum::PlanNodeEnum>,
    group_keys: Vec<String>,
    agg_exprs: Vec<String>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl AggregateNode {
    /// 创建新的聚合节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        group_keys: Vec<String>,
        agg_exprs: Vec<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = group_keys.iter().chain(agg_exprs.iter()).cloned().collect();

        let mut deps = Vec::new();
        deps.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            deps,
            group_keys,
            agg_exprs,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取分组键
    pub fn group_keys(&self) -> &[String] {
        &self.group_keys
    }

    /// 获取聚合表达式
    pub fn agg_exprs(&self) -> &[String] {
        &self.agg_exprs
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

    pub fn dependencies(&self) -> &[super::plan_node_enum::PlanNodeEnum] {
        &self.deps
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.input = dep.clone();
        self.deps.clear();
        self.deps.push(dep);
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
            agg_exprs: self.agg_exprs.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_aggregate_node_creation() {
        let start_node = super::plan_node_enum::PlanNodeEnum::Start(StartNode::new());

        let group_keys = vec!["category".to_string()];
        let agg_exprs = vec!["COUNT(*)".to_string()];

        let aggregate_node = AggregateNode::new(start_node, group_keys, agg_exprs).expect("Aggregate node should be created successfully");

        assert_eq!(aggregate_node.type_name(), "Aggregate");
        assert_eq!(aggregate_node.dependencies().len(), 1);
        assert_eq!(aggregate_node.group_keys().len(), 1);
        assert_eq!(aggregate_node.agg_exprs().len(), 1);
    }
}
