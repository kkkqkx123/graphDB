//! 过滤节点实现
//!
//! FilterNode 用于根据指定的条件过滤输入数据流

use super::plan_node_enum::PlanNodeEnum;
use crate::core::Expression;
use crate::query::context::validate::types::Variable;

/// 过滤节点
///
/// 根据指定的条件表达式过滤输入数据流
#[derive(Debug, Clone)]
pub struct FilterNode {
    id: i64,
    input: Box<PlanNodeEnum>,
    condition: Expression,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl FilterNode {
    /// 创建新的过滤节点
    pub fn new(
        input: PlanNodeEnum,
        condition: Expression,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1, // 将在后续分配
            input: Box::new(input),
            condition,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取过滤条件
    pub fn condition(&self) -> &Expression {
        &self.condition
    }

    /// 设置过滤条件
    pub fn set_condition(&mut self, condition: Expression) {
        self.condition = condition;
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "Filter"
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[Box<PlanNodeEnum>] {
        std::slice::from_ref(&self.input)
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 设置列名
    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    /// 克隆节点
    pub fn clone_plan_node(&self) -> PlanNodeEnum {
        PlanNodeEnum::Filter(Self {
            id: self.id,
            input: self.input.clone(),
            condition: self.condition.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    /// 使用新ID克隆节点
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        PlanNodeEnum::Filter(Self {
            id: new_id,
            input: self.input.clone(),
            condition: self.condition.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }
}

// 为 FilterNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for FilterNode {
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

    fn into_enum(self) -> PlanNodeEnum {
        PlanNodeEnum::Filter(self)
    }
}

// 为 FilterNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for FilterNode {
    fn input(&self) -> &PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: PlanNodeEnum) {
        self.input = Box::new(input);
    }
}

// 为 FilterNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for FilterNode {
    fn clone_plan_node(&self) -> PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Expression;

    #[test]
    fn test_filter_node_creation() {
        // 创建一个起始节点作为输入
        let start_node = crate::query::planner::plan::core::nodes::start_node::StartNode::new();
        let start_node_enum =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                start_node,
            );

        let condition = Expression::Variable("test".to_string());
        let filter_node = FilterNode::new(start_node_enum, condition)
            .expect("Filter node should be created successfully");

        assert_eq!(filter_node.type_name(), "Filter");
        assert_eq!(filter_node.dependencies().len(), 1);
    }
}
