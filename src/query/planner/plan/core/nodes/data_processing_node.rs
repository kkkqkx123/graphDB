//! 数据处理节点实现
//!
//! 包含Union、Unwind、Dedup等数据处理相关的计划节点

use crate::define_plan_node_with_deps;

define_plan_node_with_deps! {
    pub struct UnionNode {
        distinct: bool,
    }
    enum: Union
    input: SingleInputNode
}

impl UnionNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        distinct: bool,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            distinct,
            output_var: None,
            col_names,
        })
    }

    pub fn distinct(&self) -> bool {
        self.distinct
    }
}

define_plan_node_with_deps! {
    pub struct UnwindNode {
        alias: String,
        list_expression: String,
    }
    enum: Unwind
    input: SingleInputNode
}

impl UnwindNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        alias: &str,
        list_expression: &str,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let mut col_names = input.col_names().to_vec();
        col_names.push(alias.to_string());

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            alias: alias.to_string(),
            list_expression: list_expression.to_string(),
            output_var: None,
            col_names,
        })
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn list_expression(&self) -> &str {
        &self.list_expression
    }
}

define_plan_node_with_deps! {
    pub struct DedupNode {
    }
    enum: Dedup
    input: SingleInputNode
}

impl DedupNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            output_var: None,
            col_names,
        })
    }
}

define_plan_node_with_deps! {
    pub struct DataCollectNode {
        collect_kind: String,
    }
    enum: DataCollect
    input: SingleInputNode
}

impl DataCollectNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        collect_kind: &str,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            collect_kind: collect_kind.to_string(),
            output_var: None,
            col_names,
        })
    }

    pub fn collect_kind(&self) -> &str {
        &self.collect_kind
    }
}

define_plan_node_with_deps! {
    pub struct AssignNode {
        assignments: Vec<(String, String)>,
    }
    enum: Assign
    input: SingleInputNode
}

impl AssignNode {
    pub fn new(
        input: super::plan_node_enum::PlanNodeEnum,
        assignments: Vec<(String, String)>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = input.col_names().to_vec();

        Ok(Self {
            id: -1,
            input: Some(Box::new(input.clone())),
            deps: vec![Box::new(input)],
            assignments,
            output_var: None,
            col_names,
        })
    }

    pub fn assignments(&self) -> &[(String, String)] {
        &self.assignments
    }
}

/// RollUpApply节点 - 分组聚合收集
///
/// 接收左右两个输入，将右侧数据按比较列分组后收集为列表，
/// 为左侧每行返回对应的聚合结果
#[derive(Debug, Clone)]
pub struct RollUpApplyNode {
    id: i64,
    left_input: Box<super::plan_node_enum::PlanNodeEnum>,
    right_input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    left_input_var: Option<String>,
    right_input_var: Option<String>,
    compare_cols: Vec<String>,
    collect_col: Option<String>,
    output_var: Option<String>,
    col_names: Vec<String>,
}

impl RollUpApplyNode {
    pub fn new(
        left_input: super::plan_node_enum::PlanNodeEnum,
        right_input: super::plan_node_enum::PlanNodeEnum,
        compare_cols: Vec<String>,
        collect_col: Option<String>,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = left_input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(left_input.clone()));
        deps.push(Box::new(right_input.clone()));

        Ok(Self {
            id: -1,
            left_input: Box::new(left_input),
            right_input: Box::new(right_input),
            deps,
            left_input_var: None,
            right_input_var: None,
            compare_cols,
            collect_col,
            output_var: None,
            col_names,
        })
    }

    pub fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left_input
    }

    pub fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right_input
    }

    pub fn left_input_var(&self) -> Option<&String> {
        self.left_input_var.as_ref()
    }

    pub fn right_input_var(&self) -> Option<&String> {
        self.right_input_var.as_ref()
    }

    pub fn compare_cols(&self) -> &[String] {
        &self.compare_cols
    }

    pub fn collect_col(&self) -> Option<&String> {
        self.collect_col.as_ref()
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "RollUpApply"
    }

    pub fn output_var(&self) -> Option<&str> {
        self.output_var.as_deref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.deps
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.left_input = Box::new(dep.clone());
        self.deps.clear();
        self.deps.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }

    pub fn set_output_var(&mut self, var: String) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn set_left_input_var(&mut self, var: String) {
        self.left_input_var = Some(var);
    }

    pub fn set_right_input_var(&mut self, var: String) {
        self.right_input_var = Some(var);
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::RollUpApply(Self {
            id: self.id,
            left_input: self.left_input.clone(),
            right_input: self.right_input.clone(),
            deps: self.deps.clone(),
            left_input_var: self.left_input_var.clone(),
            right_input_var: self.right_input_var.clone(),
            compare_cols: self.compare_cols.clone(),
            collect_col: self.collect_col.clone(),
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::RollUpApply(cloned)
    }
}

// 为 RollUpApplyNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for RollUpApplyNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&str> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn set_output_var(&mut self, var: String) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::RollUpApply(self)
    }
}

// 为 RollUpApplyNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for RollUpApplyNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

// 为 RollUpApplyNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for RollUpApplyNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left_input
    }

    fn input_mut(&mut self) -> &mut super::plan_node_enum::PlanNodeEnum {
        &mut self.left_input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left_input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

/// PatternApply节点 - 模式匹配应用
///
/// 接收左右两个输入，根据键列判断左侧数据是否匹配右侧模式
/// 支持正向匹配（EXISTS）和反向匹配（NOT EXISTS）
#[derive(Debug, Clone)]
pub struct PatternApplyNode {
    id: i64,
    left_input: Box<super::plan_node_enum::PlanNodeEnum>,
    right_input: Box<super::plan_node_enum::PlanNodeEnum>,
    deps: Vec<Box<super::plan_node_enum::PlanNodeEnum>>,
    left_input_var: Option<String>,
    right_input_var: Option<String>,
    key_cols: Vec<String>,
    is_anti_predicate: bool,
    output_var: Option<String>,
    col_names: Vec<String>,
}

impl PatternApplyNode {
    pub fn new(
        left_input: super::plan_node_enum::PlanNodeEnum,
        right_input: super::plan_node_enum::PlanNodeEnum,
        key_cols: Vec<String>,
        is_anti_predicate: bool,
    ) -> Result<Self, crate::query::planner::planner::PlannerError> {
        let col_names = left_input.col_names().to_vec();
        let mut deps = Vec::new();
        deps.push(Box::new(left_input.clone()));
        deps.push(Box::new(right_input.clone()));

        Ok(Self {
            id: -1,
            left_input: Box::new(left_input),
            right_input: Box::new(right_input),
            deps,
            left_input_var: None,
            right_input_var: None,
            key_cols,
            is_anti_predicate,
            output_var: None,
            col_names,
        })
    }

    pub fn left_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left_input
    }

    pub fn right_input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.right_input
    }

    pub fn left_input_var(&self) -> Option<&String> {
        self.left_input_var.as_ref()
    }

    pub fn right_input_var(&self) -> Option<&String> {
        self.right_input_var.as_ref()
    }

    pub fn key_cols(&self) -> &[String] {
        &self.key_cols
    }

    pub fn is_anti_predicate(&self) -> bool {
        self.is_anti_predicate
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn type_name(&self) -> &'static str {
        "PatternApply"
    }

    pub fn output_var(&self) -> Option<&str> {
        self.output_var.as_deref()
    }

    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    pub fn dependencies(&self) -> &[Box<super::plan_node_enum::PlanNodeEnum>] {
        &self.deps
    }

    pub fn add_dependency(&mut self, dep: super::plan_node_enum::PlanNodeEnum) {
        self.left_input = Box::new(dep.clone());
        self.deps.clear();
        self.deps.push(Box::new(dep));
    }

    pub fn remove_dependency(&mut self, _id: i64) -> bool {
        false
    }

    pub fn set_output_var(&mut self, var: String) {
        self.output_var = Some(var);
    }

    pub fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    pub fn set_left_input_var(&mut self, var: String) {
        self.left_input_var = Some(var);
    }

    pub fn set_right_input_var(&mut self, var: String) {
        self.right_input_var = Some(var);
    }

    pub fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::PatternApply(Self {
            id: self.id,
            left_input: self.left_input.clone(),
            right_input: self.right_input.clone(),
            deps: self.deps.clone(),
            left_input_var: self.left_input_var.clone(),
            right_input_var: self.right_input_var.clone(),
            key_cols: self.key_cols.clone(),
            is_anti_predicate: self.is_anti_predicate,
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
        })
    }

    pub fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        super::plan_node_enum::PlanNodeEnum::PatternApply(cloned)
    }
}

// 为 PatternApplyNode 实现 PlanNode trait
impl super::plan_node_traits::PlanNode for PatternApplyNode {
    fn id(&self) -> i64 {
        self.id()
    }

    fn name(&self) -> &'static str {
        self.type_name()
    }

    fn output_var(&self) -> Option<&str> {
        self.output_var()
    }

    fn col_names(&self) -> &[String] {
        self.col_names()
    }

    fn set_output_var(&mut self, var: String) {
        self.set_output_var(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.set_col_names(names);
    }

    fn into_enum(self) -> super::plan_node_enum::PlanNodeEnum {
        super::plan_node_enum::PlanNodeEnum::PatternApply(self)
    }
}

// 为 PatternApplyNode 实现 PlanNodeClonable trait
impl super::plan_node_traits::PlanNodeClonable for PatternApplyNode {
    fn clone_plan_node(&self) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_plan_node()
    }

    fn clone_with_new_id(&self, new_id: i64) -> super::plan_node_enum::PlanNodeEnum {
        self.clone_with_new_id(new_id)
    }
}

// 为 PatternApplyNode 实现 SingleInputNode trait
impl super::plan_node_traits::SingleInputNode for PatternApplyNode {
    fn input(&self) -> &super::plan_node_enum::PlanNodeEnum {
        &self.left_input
    }

    fn input_mut(&mut self) -> &mut super::plan_node_enum::PlanNodeEnum {
        &mut self.left_input
    }

    fn set_input(&mut self, input: super::plan_node_enum::PlanNodeEnum) {
        self.left_input = Box::new(input.clone());
        self.deps.clear();
        self.deps.push(Box::new(input));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::planner::plan::core::nodes::start_node::StartNode;

    #[test]
    fn test_union_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let union_node =
            UnionNode::new(start_node, true).expect("Union node should be created successfully");

        assert_eq!(union_node.type_name(), "UnionNode");
        assert_eq!(union_node.dependencies().len(), 1);
        assert!(union_node.distinct());
    }

    #[test]
    fn test_unwind_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let unwind_node = UnwindNode::new(start_node, "item", "list")
            .expect("Unwind node should be created successfully");

        assert_eq!(unwind_node.type_name(), "UnwindNode");
        assert_eq!(unwind_node.dependencies().len(), 1);
        assert_eq!(unwind_node.alias(), "item");
        assert_eq!(unwind_node.list_expression(), "list");
    }

    #[test]
    fn test_dedup_node_creation() {
        let start_node =
            crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum::Start(
                StartNode::new(),
            );

        let dedup_node =
            DedupNode::new(start_node).expect("Dedup node should be created successfully");

        assert_eq!(dedup_node.type_name(), "DedupNode");
        assert_eq!(dedup_node.dependencies().len(), 1);
    }
}
