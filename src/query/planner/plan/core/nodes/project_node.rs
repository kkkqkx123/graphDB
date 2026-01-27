//! 投影节点实现
//!
//! ProjectNode 用于根据指定的列表达式投影输入数据流

use crate::query::context::validate::types::Variable;
use crate::query::validator::YieldColumn;

/// 投影节点
///
/// 根据指定的列表达式投影输入数据流
#[derive(Debug, Clone)]
pub struct ProjectNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    columns: Vec<YieldColumn>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl ProjectNode {
    /// 创建新的投影节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        columns: Vec<YieldColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

        Ok(Self {
            id: -1,
            input: Box::new(input),
            columns,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取投影列
    pub fn columns(&self) -> &[YieldColumn] {
        &self.columns
    }

    /// 设置投影列
    pub fn set_columns(&mut self, columns: Vec<YieldColumn>) {
        self.columns = columns;
        self.col_names = self.columns.iter().map(|col| col.alias.clone()).collect();
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Project"
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

    pub fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(dep);
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if self.input.id() == id {
            // 无法移除唯一的输入节点
            true
        } else {
            false
        }
    }

    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Project(self.clone())
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Project(cloned)
    }
}

// 为 ProjectNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for ProjectNode {
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
        super::plan_node_enum::PlanNodeEnum::Project(self)
    }
}

// 为 ProjectNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for ProjectNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input);
    }
}

// 为 ProjectNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for ProjectNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::start_node::StartNode;
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_project_node_creation() {
        // 创建一个起始节点作为输入
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let columns = vec![YieldColumn {
            expression: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];

        let project_node = ProjectNode::new(start_node, columns)
            .expect("Project node should be created successfully");

        assert_eq!(project_node.type_name(), "Project");
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }

    #[test]
    fn test_project_node_columns() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let columns = vec![
            YieldColumn {
                expression: Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expression: Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];

        let project_node = ProjectNode::new(start_node, columns)
            .expect("Project node should be created successfully");

        assert_eq!(project_node.columns().len(), 2);
        assert_eq!(project_node.columns()[0].alias, "name");
        assert_eq!(project_node.columns()[1].alias, "age");
    }
}
