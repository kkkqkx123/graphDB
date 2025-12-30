//! 排序节点实现
//!
//! SortNode 用于对输入数据进行排序操作

use crate::query::context::validate::types::Variable;

/// 排序节点
///
/// 根据指定的排序字段对输入数据进行排序
#[derive(Debug, Clone)]
pub struct SortNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    sort_items: Vec<String>,
    limit: Option<i64>,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl SortNode {
    /// 创建新的排序节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(input.clone()));

        Ok(Self {
            id: -1,
            input: Box::new(input),
            deps,
            sort_items,
            limit: None,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[String] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> Option<i64> {
        self.limit
    }

    /// 设置限制数量
    pub fn set_limit(&mut self, limit: i64) {
        self.limit = Some(limit);
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Sort"
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
        super::plan_node_enum::PlanNodeEnum::Sort(Self {
            id: self.id,
            input: self.input.clone(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Sort(cloned)
    }
}

// 为 SortNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for SortNode {
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
        super::plan_node_enum::PlanNodeEnum::Sort(self)
    }
}

// 为 SortNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for SortNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

// 为 SortNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for SortNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// 限制节点
///
/// 对输入数据进行分页限制
#[derive(Debug, Clone)]
pub struct LimitNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    offset: i64,
    count: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl LimitNode {
    /// 创建新的限制节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        offset: i64,
        count: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(input.clone()));

        Ok(Self {
            id: -1,
            input: Box::new(input),
            deps,
            offset,
            count,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取偏移量
    pub fn offset(&self) -> i64 {
        self.offset
    }

    /// 获取计数
    pub fn count(&self) -> i64 {
        self.count
    }
}

impl LimitNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "Limit"
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
        super::plan_node_enum::PlanNodeEnum::Limit(Self {
            id: self.id,
            input: self.input.clone(),
            deps: self.deps.clone(),
            offset: self.offset,
            count: self.count,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::Limit(cloned)
    }
}

// 为 LimitNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for LimitNode {
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
        super::plan_node_enum::PlanNodeEnum::Limit(self)
    }
}

// 为 LimitNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for LimitNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

// 为 LimitNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for LimitNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

/// TopN节点
///
/// 对输入数据进行排序并返回前N个结果
#[derive(Debug, Clone)]
pub struct TopNNode {
    id: i64,
    input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    sort_items: Vec<String>,
    limit: i64,
    output_var: Option<Variable>,
    col_names: Vec<String>,
    cost: f64,
}

impl TopNNode {
    /// 创建新的TopN节点
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        sort_items: Vec<String>,
        limit: i64,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(input.clone()));

        Ok(Self {
            id: -1,
            input: Box::new(input),
            deps,
            sort_items,
            limit,
            output_var: None,
            col_names,
            cost: 0.0,
        })
    }

    /// 获取排序字段
    pub fn sort_items(&self) -> &[String] {
        &self.sort_items
    }

    /// 获取限制数量
    pub fn limit(&self) -> i64 {
        self.limit
    }
}

impl TopNNode {
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "TopN"
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
        super::plan_node_enum::PlanNodeEnum::TopN(Self {
            id: self.id,
            input: self.input.clone(),
            deps: self.deps.clone(),
            sort_items: self.sort_items.clone(),
            limit: self.limit,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::TopN(cloned)
    }
}

// 为 TopNNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for TopNNode {
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
        super::plan_node_enum::PlanNodeEnum::TopN(self)
    }
}

// 为 TopNNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for TopNNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

// 为 TopNNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for TopNNode {
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
    fn test_sort_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let sort_items = vec!["name".to_string(), "age".to_string()];

        let sort_node =
            SortNode::new(start_node, sort_items).expect("SortNode creation should succeed");

        assert_eq!(sort_node.type_name(), "Sort");
        assert_eq!(sort_node.dependencies().len(), 1);
        assert_eq!(sort_node.sort_items().len(), 2);
    }

    #[test]
    fn test_limit_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let limit_node =
            LimitNode::new(start_node, 10, 100).expect("Limit node should be created successfully");

        assert_eq!(limit_node.type_name(), "Limit");
        assert_eq!(limit_node.dependencies().len(), 1);
        assert_eq!(limit_node.offset(), 10);
        assert_eq!(limit_node.count(), 100);
    }

    #[test]
    fn test_topn_node_creation() {
        let start_node = PlanNodeEnum::Start(StartNode::new());

        let sort_items = vec!["name".to_string(), "age".to_string()];
        let topn_node = TopNNode::new(start_node, sort_items, 10)
            .expect("TopN node should be created successfully");

        assert_eq!(topn_node.type_name(), "TopN");
        assert_eq!(topn_node.dependencies().len(), 1);
        assert_eq!(topn_node.sort_items().len(), 2);
        assert_eq!(topn_node.limit(), 10);
    }
}
