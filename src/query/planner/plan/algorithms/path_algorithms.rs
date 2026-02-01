//! 路径查找算法相关的计划节点
//! 包含最短路径、所有路径等算法相关的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;
use crate::query::planner::plan::core::nodes::plan_node_traits::{
    BinaryInputNode, PlanNode, SingleInputNode,
};

/// 多源最短路径计划节点
#[derive(Debug, Clone)]
pub struct MultiShortestPath {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub steps: usize,
    pub left_vid_var: String,    // 左输入顶点变量
    pub right_vid_var: String,   // 右输入顶点变量
    pub termination_var: String, // 终止条件变量
    pub single_shortest: bool,   // 是否为单最短路径
}

impl MultiShortestPath {
    pub fn new(id: i64, left: PlanNodeEnum, right: PlanNodeEnum, steps: usize) -> Self {
        let mut result = Self {
            id,
            deps: vec![left, right],
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            steps,
            left_vid_var: String::new(),
            right_vid_var: String::new(),
            termination_var: String::new(),
            single_shortest: false,
        };
        result.col_names = vec!["path".to_string()];
        result
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn left_vid_var(&self) -> &str {
        &self.left_vid_var
    }

    pub fn right_vid_var(&self) -> &str {
        &self.right_vid_var
    }

    pub fn termination_var(&self) -> &str {
        &self.termination_var
    }

    pub fn single_shortest(&self) -> bool {
        self.single_shortest
    }

    pub fn set_left_vid_var(&mut self, var: &str) {
        self.left_vid_var = var.to_string();
    }

    pub fn set_right_vid_var(&mut self, var: &str) {
        self.right_vid_var = var.to_string();
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "MultiShortestPath"
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_multi_shortest_path(self)
    }
}

impl PlanNode for MultiShortestPath {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "MultiShortestPath"
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
        PlanNodeEnum::MultiShortestPath(self)
    }
}

impl BinaryInputNode for MultiShortestPath {
    fn left_input(&self) -> &PlanNodeEnum {
        &self.deps[0]
    }

    fn right_input(&self) -> &PlanNodeEnum {
        &self.deps[1]
    }

    fn set_left_input(&mut self, input: PlanNodeEnum) {
        self.deps[0] = input;
    }

    fn set_right_input(&mut self, input: PlanNodeEnum) {
        self.deps[1] = input;
    }
}

/// BFS最短路径计划节点
#[derive(Debug, Clone)]
pub struct BFSShortest {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub steps: usize,
    pub edge_types: Vec<String>, // 边类型
    pub no_loop: bool,           // 是否无环
    pub reverse: bool,           // 是否反向搜索
}

impl BFSShortest {
    pub fn new(
        id: i64,
        dep: PlanNodeEnum,
        steps: usize,
        edge_types: Vec<String>,
        no_loop: bool,
    ) -> Self {
        Self {
            id,
            deps: vec![dep],
            output_var: None,
            col_names: vec!["path".to_string()],
            cost: 0.0,
            steps,
            edge_types,
            no_loop,
            reverse: false,
        }
    }

    pub fn set_reverse(&mut self, reverse: bool) {
        self.reverse = reverse;
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "BFSShortest"
    }

    /// 获取节点的输出变量
    pub fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    /// 设置节点的输出变量
    pub fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_bfs_shortest(self)
    }
}

impl SingleInputNode for BFSShortest {
    fn input(&self) -> &PlanNodeEnum {
        &self.deps[0]
    }

    fn set_input(&mut self, input: PlanNodeEnum) {
        self.deps[0] = input;
    }
}

impl PlanNode for BFSShortest {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "BFSShortest"
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
        PlanNodeEnum::BFSShortest(self)
    }
}

/// 所有路径计划节点
#[derive(Debug, Clone)]
pub struct AllPaths {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub steps: usize,
    pub edge_types: Vec<String>,
    pub min_hop: usize,
    pub max_hop: usize,
    pub acyclic: bool,
    pub has_step_limit: bool,
    pub limit: i64,
    pub offset: i64,
}

impl AllPaths {
    pub fn new(
        id: i64,
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        steps: usize,
        edge_types: Vec<String>,
        min_hop: usize,
        max_hop: usize,
        acyclic: bool,
    ) -> Self {
        Self {
            id,
            deps: vec![left, right],
            output_var: None,
            col_names: vec!["path".to_string()],
            cost: 0.0,
            steps,
            edge_types,
            min_hop,
            max_hop,
            acyclic,
            has_step_limit: true,
            limit: -1,
            offset: 0,
        }
    }

    pub fn min_hop(&self) -> usize {
        self.min_hop
    }

    pub fn max_hop(&self) -> usize {
        self.max_hop
    }

    pub fn is_acyclic(&self) -> bool {
        self.acyclic
    }

    pub fn limit(&self) -> i64 {
        self.limit
    }

    pub fn offset(&self) -> i64 {
        self.offset
    }

    pub fn set_limit(&mut self, limit: i64) {
        self.limit = limit;
    }

    pub fn set_offset(&mut self, offset: i64) {
        self.offset = offset;
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "AllPaths"
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

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_all_paths(self)
    }
}

impl BinaryInputNode for AllPaths {
    fn left_input(&self) -> &PlanNodeEnum {
        &self.deps[0]
    }

    fn right_input(&self) -> &PlanNodeEnum {
        &self.deps[1]
    }

    fn set_left_input(&mut self, input: PlanNodeEnum) {
        self.deps[0] = input;
    }

    fn set_right_input(&mut self, input: PlanNodeEnum) {
        self.deps[1] = input;
    }
}

impl PlanNode for AllPaths {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "AllPaths"
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
        PlanNodeEnum::AllPaths(self)
    }
}

/// 最短路径计划节点
#[derive(Debug, Clone)]
pub struct ShortestPath {
    pub id: i64,
    pub deps: Vec<PlanNodeEnum>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub edge_types: Vec<String>,
    pub max_step: usize,             // 最大步数
    pub weight_expression: Option<String>, // 权重表达式
    pub no_reverse: bool,            // 是否不允许反向
}

impl ShortestPath {
    pub fn new(
        id: i64,
        left: PlanNodeEnum,
        right: PlanNodeEnum,
        edge_types: Vec<String>,
        max_step: usize,
    ) -> Self {
        Self {
            id,
            deps: vec![left, right],
            output_var: None,
            col_names: vec!["path".to_string()],
            cost: 0.0,
            edge_types,
            max_step,
            weight_expression: None,
            no_reverse: false,
        }
    }

    pub fn max_step(&self) -> usize {
        self.max_step
    }

    pub fn set_weight_expression(&mut self, expression: String) {
        self.weight_expression = Some(expression);
    }

    pub fn weight_expression(&self) -> &Option<String> {
        &self.weight_expression
    }

    /// 获取节点的唯一ID
    pub fn id(&self) -> i64 {
        self.id
    }

    /// 获取类型名称
    pub fn type_name(&self) -> &'static str {
        "ShortestPath"
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

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_shortest_path(self)
    }
}

impl BinaryInputNode for ShortestPath {
    fn left_input(&self) -> &PlanNodeEnum {
        &self.deps[0]
    }

    fn right_input(&self) -> &PlanNodeEnum {
        &self.deps[1]
    }

    fn set_left_input(&mut self, input: PlanNodeEnum) {
        self.deps[0] = input;
    }

    fn set_right_input(&mut self, input: PlanNodeEnum) {
        self.deps[1] = input;
    }
}

impl PlanNode for ShortestPath {
    fn id(&self) -> i64 {
        self.id
    }

    fn name(&self) -> &'static str {
        "ShortestPath"
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
        PlanNodeEnum::ShortestPath(self)
    }
}
