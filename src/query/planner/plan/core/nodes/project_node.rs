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
    input: super::plan_node_enum::PlanNodeEnum,
    columns: Vec<YieldColumn>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
    dependencies_vec: Vec<super::plan_node_enum::PlanNodeEnum>, // 添加一个 Vec 来满足 trait 要求
}

impl ProjectNode {
    /// 创建新的投影节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        columns: Vec<YieldColumn>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names: Vec<String> = columns.iter().map(|col| col.alias.clone()).collect();

        let mut dependencies_vec = Vec::new();
        dependencies_vec.push(input.clone());

        Ok(Self {
            id: -1,
            input,
            columns,
            output_var: None,
            col_names,
            cost: 0.0,
            dependencies_vec,
        })
    }

    /// 获取投影列
    pub fn columns(&self) -> &[YieldColumn] {
        &self.columns
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

    pub fn dependencies(&self) -> &[super::plan_node_enum::PlanNodeEnum] {
        &self.dependencies_vec
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.input = dep.clone();
        self.dependencies_vec.clear();
        self.dependencies_vec.push(dep);
    }

    pub fn remove_dependency(&mut self, id: i64) -> bool {
        let initial_len = self.dependencies_vec.len();
        self.dependencies_vec.retain(|dep| dep.id() != id);
        let final_len = self.dependencies_vec.len();

        if initial_len != final_len {
            // 更新 input，如果原来的 input 被移除
            if self.input.id() == id {
                // 如果移除了唯一的输入节点，使用 Vec 中的第一个元素作为新的输入
                if let Some(first_dep) = self.dependencies_vec.first() {
                    self.input = first_dep.clone();
                }
            }
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
        super::plan_node_enum::PlanNodeEnum::Project(Self {
            id: self.id,
            input: self.input.clone(),
            columns: self.columns.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: self
                .dependencies_vec
                .iter()
                .map(|dep| dep.clone())
                .collect(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Project(cloned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;
    use super::super::start_node::StartNode;

    #[test]
    fn test_project_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = super::plan_node_enum::PlanNodeEnum::Start(StartNode::new());

        let columns = vec![YieldColumn {
            expr: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];

        let project_node = ProjectNode::new(start_node, columns).expect("Project node should be created successfully");

        assert_eq!(project_node.type_name(), "Project");
        assert_eq!(project_node.dependencies().len(), 1);
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }

    #[test]
    fn test_project_node_columns() {
        let start_node = super::plan_node_enum::PlanNodeEnum::Start(StartNode::new());

        let columns = vec![
            YieldColumn {
                expr: Expression::Variable("name".to_string()),
                alias: "name".to_string(),
                is_matched: false,
            },
            YieldColumn {
                expr: Expression::Variable("age".to_string()),
                alias: "age".to_string(),
                is_matched: false,
            },
        ];

        let project_node = ProjectNode::new(start_node, columns).expect("Project node should be created successfully");

        assert_eq!(project_node.columns().len(), 2);
        assert_eq!(project_node.columns()[0].alias, "name");
        assert_eq!(project_node.columns()[1].alias, "age");
    }
}
