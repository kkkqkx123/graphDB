//! 起始节点实现
//!
//! StartNode 用于表示执行计划的起始点

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
    dependencies_vec: Vec<super::plan_node_enum::PlanNodeEnum>, // 添加依赖向量
}

impl StartNode {
    /// 创建新的起始节点
    pub fn new() -> Self {
        Self {
            id: -1,
            output_var: None,
            col_names: vec![],
            cost: 0.0,
            dependencies_vec: vec![],
        }
    }

    /// 获取节点ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "Start"
    }

    /// 获取输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 获取列名
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取成本
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取依赖
    pub fn dependencies(&self) -> &[super::plan_node_enum::PlanNodeEnum] {
        &self.dependencies_vec
    }

    /// 设置输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Start(Self {
            id: self.id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: vec![],
        })
    }

    /// 使用新ID克隆节点
    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::Start(Self {
            id: new_id,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            dependencies_vec: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_node_creation() {
        let start_node = StartNode::new();

        assert_eq!(start_node.type_name(), "Start");
        assert_eq!(start_node.dependencies().len(), 0);
        assert_eq!(start_node.col_names().len(), 0);
    }

    #[test]
    fn test_start_node_mutable() {
        let mut start_node = StartNode::new();

        // 测试设置属性
        start_node.set_col_names(vec!["test".to_string()]);
        assert_eq!(start_node.col_names().len(), 1);
        assert_eq!(start_node.col_names()[0], "test");
    }
}
