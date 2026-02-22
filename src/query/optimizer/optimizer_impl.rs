//! 优化器引擎核心实现
//! 提供 Optimizer 结构体及其优化逻辑实现
//!
//! Optimizer 是优化器的主类，负责协调整个优化过程：
//! 1. 将执行计划转换为 OptGroup 结构
//! 2. 按阶段执行优化规则（从枚举直接加载）
//! 3. 生成最终的执行计划
//!
//! 本实现使用枚举+Trait 机制，提供类型安全的静态分发

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::query::context::QueryContext;
use crate::query::optimizer::core::config::{OptimizationConfig, OptimizationStats};
use crate::query::optimizer::core::OptimizationPhase;
use crate::query::optimizer::plan::{
    OptContext, OptGroup, OptGroupNode, OptRule,
};
use crate::query::optimizer::rule_enum::OptimizationRule;
use crate::query::planner::plan::{ExecutionPlan, PlanNodeEnum};
use crate::query::optimizer::OptimizerError;

/// 规则集合容器
#[derive(Debug)]
pub struct RuleSet {
    pub name: String,
    pub rules: Vec<Rc<dyn OptRule>>,
}

impl RuleSet {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rc<dyn OptRule>) {
        self.rules.push(rule);
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// 优化器主结构体
#[derive(Debug)]
pub struct Optimizer {
    pub config: OptimizationConfig,
    pub rule_sets: Vec<RuleSet>,
    pub enable_cost_model: bool,
    pub enable_rule_based: bool,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new(OptimizationConfig::default())
    }
}

impl Optimizer {
    /// 创建新的优化器实例，从枚举直接加载规则
    pub fn new(config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets: Vec::new(),
            enable_cost_model: true,
            enable_rule_based: true,
        };

        optimizer.setup_rule_sets();
        optimizer
    }

    /// 从枚举创建优化器（使用默认配置）
    pub fn from_registry() -> Self {
        let config = OptimizationConfig::default();
        Self::new(config)
    }

    /// 使用自定义配置和规则集创建优化器
    /// 如果 rule_sets 为空，则从枚举加载
    pub fn with_config(rule_sets: Vec<RuleSet>, config: OptimizationConfig) -> Self {
        let mut optimizer = Self {
            config,
            rule_sets,
            enable_cost_model: true,
            enable_rule_based: true,
        };

        if optimizer.rule_sets.is_empty() {
            optimizer.setup_rule_sets();
        }

        optimizer
    }

    /// 从枚举直接创建规则集
    fn setup_rule_sets(&mut self) {
        // 按阶段加载规则
        for phase in [
            OptimizationPhase::Rewrite,
            OptimizationPhase::Logical,
            OptimizationPhase::Physical,
        ] {
            let mut rule_set = RuleSet::new(&phase.to_string());

            // 遍历所有规则枚举，按阶段过滤
            for rule_enum in self.iter_all_rules() {
                if rule_enum.phase() == phase {
                    if let Some(rule) = rule_enum.create_instance() {
                        rule_set.add_rule(rule);
                    }
                }
            }

            if !rule_set.is_empty() {
                self.rule_sets.push(rule_set);
            }
        }
    }

    /// 遍历所有规则枚举
    fn iter_all_rules(&self) -> impl Iterator<Item = OptimizationRule> {
        [
            // 逻辑优化规则
            OptimizationRule::ProjectionPushDown,
            OptimizationRule::CombineFilter,
            OptimizationRule::CollapseProject,
            OptimizationRule::DedupElimination,
            OptimizationRule::EliminateFilter,
            OptimizationRule::EliminateRowCollect,
            OptimizationRule::RemoveNoopProject,
            OptimizationRule::EliminateAppendVertices,
            OptimizationRule::RemoveAppendVerticesBelowJoin,
            OptimizationRule::PushFilterDownAggregate,
            OptimizationRule::TopN,
            OptimizationRule::MergeGetVerticesAndProject,
            OptimizationRule::MergeGetVerticesAndDedup,
            OptimizationRule::MergeGetNbrsAndProject,
            OptimizationRule::MergeGetNbrsAndDedup,
            OptimizationRule::PushFilterDownNode,
            OptimizationRule::PushEFilterDown,
            OptimizationRule::PushVFilterDownScanVertices,
            OptimizationRule::PushFilterDownInnerJoin,
            OptimizationRule::PushFilterDownHashInnerJoin,
            OptimizationRule::PushFilterDownHashLeftJoin,
            OptimizationRule::PushFilterDownCrossJoin,
            OptimizationRule::PushFilterDownGetNbrs,
            OptimizationRule::PushFilterDownExpandAll,
            OptimizationRule::PushFilterDownAllPaths,
            OptimizationRule::EliminateEmptySetOperation,
            OptimizationRule::OptimizeSetOperationInputOrder,

            // 物理优化规则
            OptimizationRule::JoinOptimization,
            OptimizationRule::PushLimitDownGetVertices,
            OptimizationRule::PushLimitDownGetEdges,
            OptimizationRule::PushLimitDownScanVertices,
            OptimizationRule::PushLimitDownScanEdges,
            OptimizationRule::PushLimitDownIndexScan,
            OptimizationRule::ScanWithFilterOptimization,
            OptimizationRule::IndexFullScan,
            OptimizationRule::IndexScan,
            OptimizationRule::EdgeIndexFullScan,
            OptimizationRule::TagIndexFullScan,
            OptimizationRule::UnionAllEdgeIndexScan,
            OptimizationRule::UnionAllTagIndexScan,
            OptimizationRule::IndexCoveringScan,
            OptimizationRule::PushTopNDownIndexScan,
        ].into_iter()
    }

    /// 查找最优执行计划
    ///
    /// # 重构变更
    /// - 接收 Arc<QueryContext> 替代 &mut QueryContext
    pub fn find_best_plan(
        &mut self,
        query_context: Arc<QueryContext>,
        plan: ExecutionPlan,
    ) -> Result<ExecutionPlan, OptimizerError> {
        self.optimize(plan, query_context)
    }

    /// 执行优化流程
    ///
    /// # 重构变更
    /// - 接收 Arc<QueryContext> 替代 &mut QueryContext
    /// - 不再克隆 QueryContext，直接使用 Arc 共享
    pub fn optimize(
        &mut self,
        plan: ExecutionPlan,
        query_context: Arc<QueryContext>,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let mut opt_ctx = OptContext::new(query_context);

        let root_opt_node = self.build_initial_opt_group(&plan, &mut opt_ctx)?;

        let mut root_group = opt_ctx
            .find_group_by_id(root_opt_node.borrow().id)
            .ok_or(OptimizerError::group_not_found(root_opt_node.borrow().id))?
            .clone();

        self.execute_optimization(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.extract_execution_plan(&root_group, &mut opt_ctx)?;

        Ok(optimized_plan)
    }

    /// 执行优化并返回统计信息
    ///
    /// # 重构变更
    /// - 接收 Arc<QueryContext> 替代 &mut QueryContext
    /// - 不再克隆 QueryContext，直接使用 Arc 共享
    pub fn optimize_with_stats(
        &mut self,
        plan: ExecutionPlan,
        query_context: Arc<QueryContext>,
    ) -> Result<(ExecutionPlan, OptimizationStats), OptimizerError> {
        let mut opt_ctx = OptContext::new(query_context);
        let mut stats = OptimizationStats::default();

        stats.plan_nodes_before = self.count_nodes(&plan);

        let root_opt_node = self.build_initial_opt_group(&plan, &mut opt_ctx)?;

        let mut root_group = opt_ctx
            .find_group_by_id(root_opt_node.borrow().id)
            .ok_or(OptimizerError::group_not_found(root_opt_node.borrow().id))?
            .clone();

        self.execute_optimization(&mut opt_ctx, &mut root_group)?;

        let optimized_plan = self.extract_execution_plan(&root_group, &mut opt_ctx)?;

        stats.plan_nodes_after = self.count_nodes(&optimized_plan);

        Ok((optimized_plan, stats))
    }

    /// 统计计划节点数量
    fn count_nodes(&self, plan: &ExecutionPlan) -> usize {
        fn count_recursive(node: &PlanNodeEnum) -> usize {
            let mut count = 1;
            count_recursive_node(node, &mut count);
            count
        }

        fn count_recursive_node(node: &PlanNodeEnum, count: &mut usize) {
            use crate::query::planner::plan::core::nodes::SingleInputNode;

            match node {
                PlanNodeEnum::Project(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Filter(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Sort(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Limit(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::TopN(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Aggregate(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::InnerJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::LeftJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::HashInnerJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::HashLeftJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::FullOuterJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::GetNeighbors(_) | PlanNodeEnum::GetVertices(_) | PlanNodeEnum::GetEdges(_) => {
                }
                PlanNodeEnum::Sample(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Dedup(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Unwind(node) => {
                    count_recursive_node(node.input(), count);
                }
                PlanNodeEnum::Expand(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::ExpandAll(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::Traverse(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::AppendVertices(node) => {
                    for dep in node.dependencies().iter() {
                        count_recursive_node(dep, count);
                    }
                }
                PlanNodeEnum::Union(node) => {
                    for input in node.dependencies().iter() {
                        count_recursive_node(input, count);
                    }
                }
                PlanNodeEnum::CrossJoin(node) => {
                    count_recursive_node(node.left_input(), count);
                    count_recursive_node(node.right_input(), count);
                }
                PlanNodeEnum::ScanVertices(_) | PlanNodeEnum::ScanEdges(_) | PlanNodeEnum::Start(_) => {
                }
                _ => {}
            }
        }

        if let Some(root) = plan.root() {
            count_recursive(root)
        } else {
            0
        }
    }

    /// 构建初始优化组
    fn build_initial_opt_group(
        &mut self,
        plan: &ExecutionPlan,
        ctx: &mut OptContext,
    ) -> Result<Rc<RefCell<OptGroupNode>>, OptimizerError> {
        if let Some(root) = plan.root() {
            self.build_opt_group_recursive(root, ctx, vec![])
        } else {
            Err(OptimizerError::no_viable_plan())
        }
    }

    /// 递归构建优化组
    fn build_opt_group_recursive(
        &mut self,
        plan_node: &PlanNodeEnum,
        ctx: &mut OptContext,
        _current_dependencies: Vec<usize>,
    ) -> Result<Rc<RefCell<OptGroupNode>>, OptimizerError> {
        let node_id = ctx.allocate_node_id();
        let group_node = ctx.get_group_node_from_pool(node_id, plan_node.clone());

        let group_node_rc = Rc::new(RefCell::new(group_node));
        ctx.add_group_node(group_node_rc.clone())?;

        // 创建对应的 OptGroup 并注册
        let mut opt_group = OptGroup::new(node_id);
        opt_group.nodes.push(group_node_rc.clone());
        opt_group.root_group = true;
        ctx.register_group(opt_group);

        self.build_inputs_recursive(plan_node, ctx, node_id)?;

        Ok(group_node_rc)
    }

    /// 递归构建输入节点
    fn build_inputs_recursive(
        &mut self,
        plan_node: &PlanNodeEnum,
        ctx: &mut OptContext,
        parent_id: usize,
    ) -> Result<(), OptimizerError> {
        use crate::query::planner::plan::core::nodes::SingleInputNode;

        match plan_node {
            PlanNodeEnum::Project(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Filter(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Sort(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Limit(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Aggregate(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::InnerJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::LeftJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::HashInnerJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::HashLeftJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::FullOuterJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::GetNeighbors(_) => {}
            PlanNodeEnum::GetVertices(_) => {}
            PlanNodeEnum::GetEdges(_) => {}
            PlanNodeEnum::Union(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Minus(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Intersect(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::CrossJoin(node) => {
                let left_id = self.build_single_input(node.left_input(), ctx)?;
                let right_id = self.build_single_input(node.right_input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(left_id);
                    group_node.borrow_mut().add_dependency(right_id);
                }
                self.build_inputs_recursive(node.left_input(), ctx, left_id)?;
                self.build_inputs_recursive(node.right_input(), ctx, right_id)?;
            }
            PlanNodeEnum::TopN(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Sample(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Dedup(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Unwind(node) => {
                let input_id = self.build_single_input(node.input(), ctx)?;
                if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                    group_node.borrow_mut().add_dependency(input_id);
                }
                self.build_inputs_recursive(node.input(), ctx, input_id)?;
            }
            PlanNodeEnum::Expand(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::ExpandAll(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Traverse(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::AppendVertices(node) => {
                for dep in node.dependencies().iter() {
                    let dep_id = self.build_single_input(dep, ctx)?;
                    if let Some(group_node) = ctx.find_group_node_by_id(parent_id) {
                        group_node.borrow_mut().add_dependency(dep_id);
                    }
                    self.build_inputs_recursive(dep, ctx, dep_id)?;
                }
            }
            PlanNodeEnum::Start(_) | PlanNodeEnum::IndexScan(_)
            | PlanNodeEnum::ScanVertices(_) | PlanNodeEnum::ScanEdges(_) | PlanNodeEnum::EdgeIndexScan(_)
            | PlanNodeEnum::Argument(_) | PlanNodeEnum::Loop(_) | PlanNodeEnum::PassThrough(_)
            | PlanNodeEnum::Select(_) | PlanNodeEnum::DataCollect(_) | PlanNodeEnum::PatternApply(_)
            | PlanNodeEnum::RollUpApply(_) | PlanNodeEnum::Assign(_)
            | PlanNodeEnum::MultiShortestPath(_) | PlanNodeEnum::BFSShortest(_) 
            | PlanNodeEnum::AllPaths(_) | PlanNodeEnum::ShortestPath(_)
            | PlanNodeEnum::CreateSpace(_) | PlanNodeEnum::DropSpace(_) | PlanNodeEnum::DescSpace(_)
            | PlanNodeEnum::ShowSpaces(_) | PlanNodeEnum::CreateTag(_) | PlanNodeEnum::AlterTag(_)
            | PlanNodeEnum::DescTag(_) | PlanNodeEnum::DropTag(_) | PlanNodeEnum::ShowTags(_)
            | PlanNodeEnum::CreateEdge(_) | PlanNodeEnum::AlterEdge(_) | PlanNodeEnum::DescEdge(_)
            | PlanNodeEnum::DropEdge(_) | PlanNodeEnum::ShowEdges(_)
            | PlanNodeEnum::CreateTagIndex(_) | PlanNodeEnum::DropTagIndex(_) | PlanNodeEnum::DescTagIndex(_)
            | PlanNodeEnum::ShowTagIndexes(_) | PlanNodeEnum::CreateEdgeIndex(_) | PlanNodeEnum::DropEdgeIndex(_)
            | PlanNodeEnum::DescEdgeIndex(_) | PlanNodeEnum::ShowEdgeIndexes(_)
            | PlanNodeEnum::RebuildTagIndex(_) | PlanNodeEnum::RebuildEdgeIndex(_)
            | PlanNodeEnum::CreateUser(_) | PlanNodeEnum::AlterUser(_) | PlanNodeEnum::DropUser(_)
            | PlanNodeEnum::ChangePassword(_)
            | PlanNodeEnum::InsertVertices(_) | PlanNodeEnum::InsertEdges(_) => {
                // 这些节点在当前上下文中没有输入需要处理
            }
        }

        Ok(())
    }

    /// 构建单个输入节点
    fn build_single_input(
        &mut self,
        input: &PlanNodeEnum,
        ctx: &mut OptContext,
    ) -> Result<usize, OptimizerError> {
        let node_id = ctx.allocate_node_id();
        let group_node = ctx.get_group_node_from_pool(node_id, input.clone());
        let group_node_rc = Rc::new(RefCell::new(group_node));
        ctx.add_group_node(group_node_rc.clone())?;
        Ok(node_id)
    }

    /// 执行优化流程
    fn execute_optimization(
        &mut self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
    ) -> Result<(), OptimizerError> {
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Rewrite)?;
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Logical)?;
        self.execute_phase_optimization(ctx, root_group, OptimizationPhase::Physical)?;

        Ok(())
    }

    /// 执行指定阶段的优化
    /// 从 rule_sets 中查找对应阶段的规则集并应用
    fn execute_phase_optimization(
        &mut self,
        ctx: &mut OptContext,
        root_group: &mut OptGroup,
        phase: OptimizationPhase,
    ) -> Result<(), OptimizerError> {
        // 查找对应阶段的规则集
        let phase_name = phase.to_string();
        let phase_rules: Vec<Rc<dyn OptRule>> = self
            .rule_sets
            .iter()
            .filter(|rs| rs.name == phase_name)
            .flat_map(|rs| rs.rules.iter().cloned())
            .collect();

        if phase_rules.is_empty() {
            return Ok(());
        }

        let max_rounds = self.config.max_iteration_rounds;
        let min_rounds = self.config.min_iteration_rounds;
        let stable_threshold = self.config.stable_threshold;
        let enable_adaptive = self.config.enable_adaptive_iteration;

        let mut round = 0;
        let mut stable_count = 0;

        while round < max_rounds {
            let before_nodes = root_group.nodes.len();

            ctx.set_changed(false);

            // 直接遍历阶段规则，无需字符串匹配
            for rule in &phase_rules {
                self.apply_rule(ctx, &*root_group, &**rule)?;
                self.clear_visited(ctx);
            }

            round += 1;

            if ctx.changed() {
                stable_count = 0;
            } else {
                stable_count += 1;
            }

            let _last_changes = root_group.nodes.len().saturating_sub(before_nodes);

            if enable_adaptive
                && round >= min_rounds
                && stable_count >= stable_threshold
                && _last_changes == 0
            {
                break;
            }
        }

        Ok(())
    }

    /// 清除访问标记
    fn clear_visited(&mut self, ctx: &mut OptContext) {
        let group_ids: Vec<usize> = {
            let group_map = ctx.group_map_mut();
            group_map.keys().cloned().collect()
        };

        for group_id in group_ids {
            if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
                group.set_visited(false);
                ctx.register_group(group);
            }
        }
    }

    /// 应用规则
    fn apply_rule(
        &mut self,
        ctx: &mut OptContext,
        root_group: &OptGroup,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        let root_group_id = root_group.id;
        self.explore_until_max_round(ctx, root_group_id, rule)
    }

    /// 递归探索直到达到最大轮次
    fn explore_until_max_round(
        &mut self,
        ctx: &mut OptContext,
        group_id: usize,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        const MAX_EXPLORATION_ROUND: i32 = 128;
        let mut max_round = MAX_EXPLORATION_ROUND;
        
        while !self.is_group_explored(ctx, group_id, rule) {
            if max_round <= 0 {
                break;
            }
            max_round -= 1;
            self.explore_group(ctx, group_id, rule)?;
        }
        
        Ok(())
    }

    /// 检查组是否已探索
    fn is_group_explored(&self, ctx: &OptContext, group_id: usize, rule: &dyn OptRule) -> bool {
        if let Some(group) = ctx.find_group_by_id(group_id) {
            group.is_explored(rule.name())
        } else {
            true
        }
    }

    /// 探索组
    fn explore_group(
        &mut self,
        ctx: &mut OptContext,
        group_id: usize,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        if self.is_group_explored(ctx, group_id, rule) {
            return Ok(());
        }

        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            group.set_explored(rule.name());
            ctx.register_group(group);
        }

        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            if group.is_visited() {
                return Ok(());
            }
            group.set_visited(true);
            ctx.register_group(group);
        }

        let nodes: Vec<Rc<RefCell<OptGroupNode>>> = {
            if let Some(group) = ctx.find_group_by_id(group_id) {
                group.nodes.iter().cloned().collect()
            } else {
                return Ok(());
            }
        };

        for node_rc in nodes {
            if self.is_node_explored(&node_rc.borrow(), rule) {
                continue;
            }

            self.explore_group_node(ctx, &node_rc, rule)?;

            if !rule.pattern().matches(&node_rc.borrow().plan_node) {
                continue;
            }

            match rule.apply(ctx, &node_rc) {
                Ok(Some(result)) => {
                    let node_id = node_rc.borrow().id;
                    let node_dependencies = node_rc.borrow().dependencies.clone();
                    let has_new_nodes = !result.new_group_nodes.is_empty();

                    if result.erase_curr || result.erase_all {
                        if let Some(mut group) = ctx.find_group_by_id_mut(node_id) {
                            group.nodes.retain(|n| n.borrow().id != node_id);
                            ctx.register_group(group);
                        }
                    }

                    for new_node in result.new_group_nodes {
                        let new_node_id = new_node.borrow().id;
                        if let Some(mut group) = ctx.find_group_by_id_mut(new_node_id) {
                            if !group.nodes.iter().any(|n| n.borrow().id == new_node_id) {
                                group.add_node(new_node);
                            }
                            ctx.register_group(group);
                        } else {
                            let mut new_group = OptGroup::new(new_node_id);
                            new_group.add_node(new_node);
                            ctx.register_group(new_group);
                        }
                    }

                    for &new_dep in &result.new_dependencies {
                        if !node_dependencies.contains(&new_dep) {
                            let mut node_mut = node_rc.borrow_mut();
                            node_mut.dependencies.push(new_dep);
                        }
                    }

                    ctx.set_changed(true);

                    if has_new_nodes {
                        self.set_group_unexplored(ctx, group_id, rule);
                    }
                }
                Ok(None) => {
                    let mut node_mut = node_rc.borrow_mut();
                    node_mut.set_explored(rule.name());
                }
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// 探索组节点
    fn explore_group_node(
        &mut self,
        ctx: &mut OptContext,
        node: &Rc<RefCell<OptGroupNode>>,
        rule: &dyn OptRule,
    ) -> Result<(), OptimizerError> {
        if self.is_node_explored(&node.borrow(), rule) {
            return Ok(());
        }

        let mut node_mut = node.borrow_mut();
        node_mut.set_explored(rule.name());
        drop(node_mut);

        let dependencies: Vec<usize> = node.borrow().dependencies.clone();
        let bodies: Vec<usize> = node.borrow().bodies.clone();

        for dep_id in dependencies {
            self.explore_until_max_round(ctx, dep_id, rule)?;
        }

        for body_id in bodies {
            self.explore_until_max_round(ctx, body_id, rule)?;
        }

        Ok(())
    }

    /// 检查节点是否已探索
    fn is_node_explored(&self, node: &OptGroupNode, rule: &dyn OptRule) -> bool {
        node.is_explored(rule.name())
    }

    /// 设置组为未探索状态
    fn set_group_unexplored(&mut self, ctx: &mut OptContext, group_id: usize, rule: &dyn OptRule) {
        if let Some(mut group) = ctx.find_group_by_id_mut(group_id) {
            if group.is_visited() {
                return;
            }
            group.set_visited(true);
            group.set_unexplored(rule.name());

            let node_ids: Vec<usize> = group.nodes.iter().map(|n| n.borrow().id).collect();
            let dependencies = group.get_all_dependencies();
            let bodies = group.get_all_bodies();

            ctx.register_group(group);

            for node_id in node_ids {
                if let Some(node_group) = ctx.find_group_by_id(node_id) {
                    if let Some(node_rc) = node_group.get_node_by_id(node_id) {
                        let mut node_mut = node_rc.borrow_mut();
                        node_mut.set_unexplored(rule.name());
                    }
                }
            }

            for dep_id in dependencies {
                self.set_group_unexplored(ctx, dep_id, rule);
            }

            for body_id in bodies {
                self.set_group_unexplored(ctx, body_id, rule);
            }
        }
    }

    /// 提取执行计划
    fn extract_execution_plan(
        &self,
        root_group: &OptGroup,
        ctx: &mut OptContext,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let root_node = root_group
            .get_min_cost_group_node()
            .ok_or(OptimizerError::no_viable_plan())?;

        self.build_execution_plan_recursive(&root_node, ctx)
    }

    /// 递归构建执行计划
    fn build_execution_plan_recursive(
        &self,
        opt_node: &Rc<RefCell<OptGroupNode>>,
        ctx: &mut OptContext,
    ) -> Result<ExecutionPlan, OptimizerError> {
        let opt_node_borrowed = opt_node.borrow();

        let mut inputs: Vec<ExecutionPlan> = Vec::new();

        for &dep_id in &opt_node_borrowed.dependencies {
            if let Some(dep_group) = ctx.find_group_by_id(dep_id) {
                if let Some(dep_node) = dep_group.get_min_cost_group_node() {
                    let input_plan = self.build_execution_plan_recursive(&dep_node, ctx)?;
                    inputs.push(input_plan);
                }
            }
        }

        let plan_node = opt_node_borrowed.plan_node.clone();

        drop(opt_node_borrowed);

        let root_plan_node = plan_node;

        let plan = ExecutionPlan::new(Some(root_plan_node));

        Ok(plan)
    }

    /// 添加规则集
    pub fn add_rule_set(&mut self, rule_set: RuleSet) {
        self.rule_sets.push(rule_set);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let optimizer = Optimizer::default();
        assert!(!optimizer.rule_sets.is_empty());
    }

    #[test]
    fn test_rule_set_creation() {
        let rule_set = RuleSet::new("test");
        assert_eq!(rule_set.name, "test");
        assert!(rule_set.is_empty());
    }

    #[test]
    fn test_optimizer_from_registry() {
        let optimizer = Optimizer::from_registry();
        assert!(!optimizer.rule_sets.is_empty());
    }
}
