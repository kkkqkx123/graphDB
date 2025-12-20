//! 路径查找算法相关的计划节点
//! 包含最短路径、所有路径等算法相关的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::{
    plan_node_traits::{
        PlanNode, PlanNodeClonable, PlanNodeDependencies, PlanNodeDependenciesExt,
        PlanNodeIdentifiable, PlanNodeMutable, PlanNodeProperties, PlanNodeVisitable,
    },
    PlanNodeKind, PlanNodeVisitError, PlanNodeVisitor,
};
use std::sync::Arc;

/// 多源最短路径计划节点
#[derive(Debug)]
pub struct MultiShortestPath {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
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
    pub fn new(id: i64, left: Arc<dyn PlanNode>, right: Arc<dyn PlanNode>, steps: usize) -> Self {
        let mut result = Self {
            id,
            kind: PlanNodeKind::MultiShortestPath,
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
}

impl Clone for MultiShortestPath {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            steps: self.steps,
            left_vid_var: self.left_vid_var.clone(),
            right_vid_var: self.right_vid_var.clone(),
            termination_var: self.termination_var.clone(),
            single_shortest: self.single_shortest,
        }
    }
}

impl PlanNodeIdentifiable for MultiShortestPath {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for MultiShortestPath {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for MultiShortestPath {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(index) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(index);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for MultiShortestPath {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for MultiShortestPath {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for MultiShortestPath {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for MultiShortestPath {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for MultiShortestPath {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// BFS最短路径计划节点
#[derive(Debug)]
pub struct BFSShortest {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
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
        dep: Arc<dyn PlanNode>,
        steps: usize,
        edge_types: Vec<String>,
        no_loop: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::BFSShortest,
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
}

impl Clone for BFSShortest {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            steps: self.steps,
            edge_types: self.edge_types.clone(),
            no_loop: self.no_loop,
            reverse: self.reverse,
        }
    }
}

impl PlanNodeIdentifiable for BFSShortest {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for BFSShortest {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for BFSShortest {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for BFSShortest {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for BFSShortest {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for BFSShortest {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for BFSShortest {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for BFSShortest {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 所有路径计划节点
#[derive(Debug)]
pub struct AllPaths {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub steps: usize,
    pub edge_types: Vec<String>,
    pub min_hop: usize,       // 最小跳数
    pub max_hop: usize,       // 最大跳数
    pub acyclic: bool,        // 是否无环
    pub has_step_limit: bool, // 是否有步数限制
}

impl AllPaths {
    pub fn new(
        id: i64,
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        steps: usize,
        edge_types: Vec<String>,
        min_hop: usize,
        max_hop: usize,
        acyclic: bool,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::AllPaths,
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
}

impl Clone for AllPaths {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            steps: self.steps,
            edge_types: self.edge_types.clone(),
            min_hop: self.min_hop,
            max_hop: self.max_hop,
            acyclic: self.acyclic,
            has_step_limit: self.has_step_limit,
        }
    }
}

impl PlanNodeIdentifiable for AllPaths {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for AllPaths {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for AllPaths {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for AllPaths {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for AllPaths {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for AllPaths {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for AllPaths {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for AllPaths {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 最短路径计划节点
#[derive(Debug)]
pub struct ShortestPath {
    pub id: i64,
    pub kind: PlanNodeKind,
    pub deps: Vec<Arc<dyn PlanNode>>,
    pub output_var: Option<Variable>,
    pub col_names: Vec<String>,
    pub cost: f64,
    pub edge_types: Vec<String>,
    pub max_step: usize,             // 最大步数
    pub weight_expr: Option<String>, // 权重表达式
    pub no_reverse: bool,            // 是否不允许反向
}

impl ShortestPath {
    pub fn new(
        id: i64,
        left: Arc<dyn PlanNode>,
        right: Arc<dyn PlanNode>,
        edge_types: Vec<String>,
        max_step: usize,
    ) -> Self {
        Self {
            id,
            kind: PlanNodeKind::ShortestPath,
            deps: vec![left, right],
            output_var: None,
            col_names: vec!["path".to_string()],
            cost: 0.0,
            edge_types,
            max_step,
            weight_expr: None,
            no_reverse: false,
        }
    }

    pub fn max_step(&self) -> usize {
        self.max_step
    }

    pub fn set_weight_expr(&mut self, expr: String) {
        self.weight_expr = Some(expr);
    }

    pub fn weight_expr(&self) -> &Option<String> {
        &self.weight_expr
    }
}

impl Clone for ShortestPath {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            kind: self.kind.clone(),
            deps: Vec::new(), // 克隆时不包含依赖
            output_var: self.output_var.clone(),
            col_names: self.col_names.clone(),
            cost: self.cost,
            edge_types: self.edge_types.clone(),
            max_step: self.max_step,
            weight_expr: self.weight_expr.clone(),
            no_reverse: self.no_reverse,
        }
    }
}

impl PlanNodeIdentifiable for ShortestPath {
    fn id(&self) -> i64 {
        self.id
    }

    fn kind(&self) -> PlanNodeKind {
        self.kind.clone()
    }
}

impl PlanNodeProperties for ShortestPath {
    fn output_var(&self) -> Option<&Variable> {
        self.output_var.as_ref()
    }

    fn col_names(&self) -> &[String] {
        &self.col_names
    }

    fn cost(&self) -> f64 {
        self.cost
    }
}

impl PlanNodeDependencies for ShortestPath {
    fn dependencies(&self) -> Vec<Arc<dyn PlanNode>> {
        self.deps.clone()
    }

    fn add_dependency(&mut self, dep: Arc<dyn PlanNode>) {
        self.deps.push(dep);
    }

    fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
    }
}

impl PlanNodeDependenciesExt for ShortestPath {
    fn with_dependencies<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&[Arc<dyn PlanNode>]) -> R,
    {
        f(&self.deps)
    }
}

impl PlanNodeMutable for ShortestPath {
    fn set_output_var(&mut self, var: Variable) {
        self.output_var = Some(var);
    }

    fn set_col_names(&mut self, names: Vec<String>) {
        self.col_names = names;
    }
}

impl PlanNodeClonable for ShortestPath {
    fn clone_plan_node(&self) -> Arc<dyn PlanNode> {
        Arc::new(self.clone())
    }

    fn clone_with_new_id(&self, new_id: i64) -> Arc<dyn PlanNode> {
        let mut cloned = self.clone();
        cloned.id = new_id;
        Arc::new(cloned)
    }
}

impl PlanNodeVisitable for ShortestPath {
    fn accept(&self, visitor: &mut dyn PlanNodeVisitor) -> Result<(), PlanNodeVisitError> {
        visitor.pre_visit()?;
        visitor.post_visit()?;
        Ok(())
    }
}

impl PlanNode for ShortestPath {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
