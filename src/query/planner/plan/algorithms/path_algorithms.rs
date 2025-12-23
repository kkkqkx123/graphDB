//! 路径查找算法相关的计划节点
//! 包含最短路径、所有路径等算法相关的计划节点

use crate::query::context::validate::types::Variable;
use crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeEnum;

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

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(index) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(index);
            true
        } else {
            false
        }
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
        // 这里需要创建一个 PlanNodeEnum::MultiShortestPath，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::MultiShortestPath，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_multi_shortest_path(self)
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

    /// 获取列名列表
    pub fn col_names(&self) -> &[String] {
        &self.col_names
    }

    /// 获取节点的成本估计值
    pub fn cost(&self) -> f64 {
        self.cost
    }

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
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
        // 这里需要创建一个 PlanNodeEnum::BFSShortest，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::BFSShortest，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_bfs_shortest(self)
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
    pub min_hop: usize,       // 最小跳数
    pub max_hop: usize,       // 最大跳数
    pub acyclic: bool,        // 是否无环
    pub has_step_limit: bool, // 是否有步数限制
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

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
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
        // 这里需要创建一个 PlanNodeEnum::AllPaths，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::AllPaths，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_all_paths(self)
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
    pub weight_expr: Option<String>, // 权重表达式
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

    /// 获取节点的依赖节点列表
    pub fn dependencies(&self) -> &[PlanNodeEnum] {
        &self.deps
    }

    /// 添加依赖节点
    pub fn add_dependency(&mut self, dep: PlanNodeEnum) {
        self.deps.push(dep);
    }

    /// 移除依赖节点
    pub fn remove_dependency(&mut self, id: i64) -> bool {
        if let Some(pos) = self.deps.iter().position(|dep| dep.id() == id) {
            self.deps.remove(pos);
            true
        } else {
            false
        }
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
        // 这里需要创建一个 PlanNodeEnum::ShortestPath，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 克隆节点并分配新的ID
    pub fn clone_with_new_id(&self, new_id: i64) -> PlanNodeEnum {
        let mut cloned = self.clone();
        cloned.id = new_id;
        // 这里需要创建一个 PlanNodeEnum::ShortestPath，但需要先在 PlanNodeEnum 中添加这个变体
        // 暂时返回一个 StartNode 作为占位符
        PlanNodeEnum::Start(crate::query::planner::plan::core::nodes::start_node::StartNode::new())
    }

    /// 使用访问者模式访问节点
    pub fn accept<V>(&self, visitor: &mut V) -> V::Result
    where
        V: crate::query::planner::plan::core::nodes::plan_node_enum::PlanNodeVisitor,
    {
        visitor.visit_shortest_path(self)
    }
}
