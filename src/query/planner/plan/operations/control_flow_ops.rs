//! 控制流操作节点
//! 包含Start、Argument、Select、Loop等控制流相关的计划节点

use crate::query::planner::plan::core::{PlanNode as BasePlanNode, PlanNodeKind, PlanNodeVisitor, PlanNodeVisitError};
use crate::query::validator::Variable;

/// Start节点 - 一个特殊的叶子节点，帮助调度器正常工作
#[derive(Debug)]
pub struct Start {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl Start {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Start,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl Clone for Start {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl BasePlanNode for Start {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_start(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Argument节点 - 用于从另一个已执行的操作中获取命名别名
#[derive(Debug)]
pub struct Argument {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub var: String, // 参数变量
}

impl Argument {
    pub fn new(id: i64, var: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Argument,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            var: var.to_string(),
        }
    }
}

impl Clone for Argument {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            var: self.var.clone(),
        }
    }
}

impl BasePlanNode for Argument {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_argument(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// 从 logic_nodes.rs 的 StartNode 与 ArgumentNode 合并进来
/// StartNode - 逻辑节点中的Start节点定义，与other_ops.rs中的Start保持一致
#[derive(Debug)]
pub struct StartNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub output_var_name: Option<String>,  // 逻辑节点的特有属性
}

impl StartNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Start,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            output_var_name: None,
        }
    }
}

impl Clone for StartNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            output_var_name: self.output_var_name.clone(),
        }
    }
}

impl BasePlanNode for StartNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_start_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// ArgumentNode - 逻辑节点中的Argument节点定义，与other_ops.rs中的Argument保持一致
#[derive(Debug)]
pub struct ArgumentNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub alias: String,  // 参数别名
    pub input_vertex_required: bool,  // 是否需要输入顶点
}

impl ArgumentNode {
    pub fn new(id: i64, alias: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Argument,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            alias: alias.to_string(),
            input_vertex_required: true,
        }
    }

    pub fn set_input_vertex_required(&mut self, required: bool) {
        self.input_vertex_required = required;
    }

    pub fn is_input_vertex_required(&self) -> bool {
        self.input_vertex_required
    }

    pub fn get_alias(&self) -> &str {
        &self.alias
    }
}

impl Clone for ArgumentNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            alias: self.alias.clone(),
            input_vertex_required: self.input_vertex_required,
        }
    }
}

impl BasePlanNode for ArgumentNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.visit_argument_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 二元选择节点 - 用于Select和Loop节点的基础
#[derive(Debug)]
pub struct BinarySelectNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub condition: String,  // 选择条件
}

impl BinarySelectNode {
    pub fn new(id: i64, kind: PlanNodeKind, condition: &str) -> Self {
        Self {
            id,
            kind,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            condition: condition.to_string(),
        }
    }
}

impl Clone for BinarySelectNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            condition: self.condition.clone(),
        }
    }
}

impl BasePlanNode for BinarySelectNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        // Since there's no specific visitor method for binary select, we'll call visit_plan_node
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Select节点 - 在运行时选择if分支或else分支
#[derive(Debug)]
pub struct SelectNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub condition: String,
    pub if_branch: Option<Box<dyn BasePlanNode>>,  // IF分支
    pub else_branch: Option<Box<dyn BasePlanNode>>, // ELSE分支
}

impl SelectNode {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Select,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            condition: condition.to_string(),
            if_branch: None,
            else_branch: None,
        }
    }

    pub fn set_if_branch(&mut self, branch: Box<dyn BasePlanNode>) {
        self.if_branch = Some(branch);
    }

    pub fn set_else_branch(&mut self, branch: Box<dyn BasePlanNode>) {
        self.else_branch = Some(branch);
    }

    pub fn if_branch(&self) -> &Option<Box<dyn BasePlanNode>> {
        &self.if_branch
    }

    pub fn else_branch(&self) -> &Option<Box<dyn BasePlanNode>> {
        &self.else_branch
    }
}

impl Clone for SelectNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            condition: self.condition.clone(),
            if_branch: self.if_branch.as_ref().map(|node| node.clone_plan_node()),
            else_branch: self.else_branch.as_ref().map(|node| node.clone_plan_node()),
        }
    }
}

impl BasePlanNode for SelectNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        // Since there's no specific visitor method for select, we'll call visit_plan_node
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Loop节点 - 在运行时多次执行分支
#[derive(Debug)]
pub struct LoopNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub condition: String,
    pub body: Option<Box<dyn BasePlanNode>>,  // 循环体
}

impl LoopNode {
    pub fn new(id: i64, condition: &str) -> Self {
        Self {
            id,
            kind: PlanNodeKind::Loop,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
            condition: condition.to_string(),
            body: None,
        }
    }

    pub fn set_body(&mut self, body: Box<dyn BasePlanNode>) {
        self.body = Some(body);
    }

    pub fn body(&self) -> &Option<Box<dyn BasePlanNode>> {
        &self.body
    }
}

impl Clone for LoopNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            condition: self.condition.clone(),
            body: self.body.as_ref().map(|node| node.clone_plan_node()),
        }
    }
}

impl BasePlanNode for LoopNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        // Since there's no specific visitor method for loop, we'll call visit_plan_node
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// PassThrough节点 - 用于透传情况的节点
#[derive(Debug)]
pub struct PassThroughNode {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Box<dyn BasePlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
}

impl PassThroughNode {
    pub fn new(id: i64) -> Self {
        Self {
            id,
            kind: PlanNodeKind::PassThrough,
            deps: Vec::new(),
            output_var: None,
            col_names: Vec::new(),
            cost: 0.0,
        }
    }
}

impl Clone for PassThroughNode {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
        }
    }
}

impl BasePlanNode for PassThroughNode {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }

    fn dependencies(&self) -> &Vec<Box<dyn BasePlanNode>> {
        &self.deps
    }

    fn output_var(&self) -> &Option<Variable> {
        &self.output_var
    }

    fn col_names(&self) -> &Vec<String> {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }

    fn clone_plan_node(&self) -> Box<dyn BasePlanNode> {
        Box::new(self.clone())
    }

    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        // Since there's no specific visitor method for pass through, we'll call visit_plan_node
        visitor.visit_plan_node(self)?;
        visitor.post_visit()?;
        Ok(())
    }

    fn set_dependencies(&mut self, deps: Vec<Box<dyn BasePlanNode>>) {
        self.deps = deps;
    }

    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }

    fn set_cost(&mut self, cost: f64) {
        self.cost = cost;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}