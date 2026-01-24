//! 起始节点实现
//!
//! StartNode 用于表示执行计划的起始点

use super::plan_node_enum::PlanNodeEnum;
use super::plan_node_traits::{PlanNode, PlanNodeClonable, ZeroInputNode};
use crate::query::context::validate::types::Variable;

/// 起始节点
///
/// 表示执行计划的起始点，没有输入依赖
#[derive(Debug, Clone)]
pub struct StartNode {
    id: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl StartNode {
    /// 创建新的起始节点
    pub fn new() -> Self {
        Self {
            id: -1,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
        }
    }
}

impl PlanNode for StartNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "Start"
    }

    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::Start(self)
    }
}

impl ZeroInputNode for StartNode {}

impl PlanNodeClonable for StartNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::Start(Self {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        PlanNodeEnum::Start(Self {
            id: new_id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_node_creation() {
        let start_node = StartNode::new();

        assert_eq!(start_node.name(), "Start");
        assert_eq!(start_node.input_count(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }

    #[test]
    fn test_start_node_mutable() {
        let mut start_node = StartNode::new();

        start_node.set_col_names(vec!["test".to_string()]);
        assert_eq!(start_node.col_names().len(), 1);
        assert_eq!(start_node.col_names()[0], "test");
    }

    #[test]
    fn test_start_node_traits() {
        let start_node = StartNode::new();

        assert_eq!(start_node.id(), -1);
        assert_eq!(start_node.cost(), 0.0);
        assert!(start_node.output_var().is_none());
    }

    #[test]
    fn test_start_node_clone() {
        let mut start_node = StartNode::new();
        start_node.set_col_names(vec!["col1".to_string(), "col2".to_string()]);

        let cloned = start_node.clone_plan_node();
        assert_eq!(cloned.name(), "Start");
        assert_eq!(cloned.col_names().len(), 2);

        let cloned_with_id = start_node.clone_with_new_id(100);
        assert_eq!(cloned_with_id.id(), 100);
    }
}
