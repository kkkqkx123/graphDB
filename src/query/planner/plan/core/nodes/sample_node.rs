//! 采样节点实现
//!
//! SampleNode 用于对输入数据进行随机采样操作

use crate::query::context::validate::types::Variable;

#[derive(Debug, Clone)]
pub struct SampleNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    count: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl SampleNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        count: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(input.clone()));

        Ok(Self {
            id: -1,
            input: Box::new(input),
            deps,
            count,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    pub fn count(&self) -> i64 {
        self.count
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Sample"
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
        false
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Sample(Self {
            id: self.id,
            input: self.input.clone(),
            deps: self.deps.clone(),
            count: self.count,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Sample(cloned)
    }
}

impl super::plan_node_traits::PlanNode for SampleNode {
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
        super::plan_node_enum::PlanNodeEnum::Sample(self)
    }
}

impl super::plan_node_traits::SingleInputNode for SampleNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

impl super::plan_node_traits::PlanNodeClonable for SampleNode {
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
    use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_sample_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let sample_node = SampleNode::new(start_node, 10)
            .expect("SampleNode creation should succeed");

        assert_eq!(sample_node.type_name(), "Sample");
        assert_eq!(sample_node.dependencies().len(), 1);
        assert_eq!(sample_node.count(), 10);
    }
}
