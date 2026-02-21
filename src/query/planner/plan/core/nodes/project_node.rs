//! 投影节点实现
//!
//! ProjectNode 用于根据指定的列表达式投影输入数据流

use crate::define_plan_node_with_deps;
use crate::core::YieldColumn;

define_plan_node_with_deps! {
    pub struct ProjectNode {
        columns: Vec<YieldColumn>,
    }
    enum: Project
    input: SingleInputNode
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
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_project_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                crate::query::planner::plan::core::nodes::start_node::StartNode::new(),
            );

        let columns = vec![YieldColumn {
            expression: Expression::Variable("test".to_string()),
            alias: "test".to_string(),
            is_matched: false,
        }];

        let project_node = ProjectNode::new(start_node, columns)
            .expect("Project node should be created successfully");

        assert_eq!(project_node.type_name(), "ProjectNode");
        assert_eq!(project_node.col_names().len(), 1);
        assert_eq!(project_node.col_names()[0], "test");
    }

    #[test]
    fn test_project_node_columns() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                crate::query::planner::plan::core::nodes::start_node::StartNode::new(),
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
