//! 计划重写器实现
//!
//! 管理所有启发式重写规则，按顺序应用到计划树。
//! 使用静态分发（枚举）替代动态分发（Box<dyn>），提供更好的性能。
//!
//! # 性能优势
//!
//! - 无动态分发开销（无虚函数表查找）
//! - 无堆分配（规则存储在栈上）
//! - 更好的缓存局部性
//! - 编译器可以内联优化

use crate::query::optimizer::plan::{OptContext, OptGroupNode, TransformResult};
use crate::query::optimizer::OptimizerError;
use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::rewrite::rewrite_rule::RewriteError;
use crate::query::planner::rewrite::rule_enum::{RewriteRule, RuleRegistry};
use crate::query::QueryContext;
use std::rc::Rc;
use std::cell::RefCell;
use std::sync::Arc;

/// 计划重写器
///
/// 管理所有启发式重写规则，按顺序应用。
/// 规则按添加顺序执行，每个规则可能被多次应用直到不再产生变化。
///
/// 使用静态分发枚举存储规则，避免动态分发的开销。
#[derive(Debug)]
pub struct PlanRewriter {
    /// 已注册的规则列表（静态分发）
    rules: Vec<RewriteRule>,
    /// 最大迭代次数，防止无限循环
    max_iterations: usize,
}

impl PlanRewriter {
    /// 创建新的计划重写器
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            max_iterations: 100,
        }
    }

    /// 从规则注册表创建
    pub fn from_registry(registry: RuleRegistry) -> Self {
        Self {
            rules: registry.into_vec(),
            max_iterations: 100,
        }
    }

    /// 设置最大迭代次数
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// 添加规则
    pub fn add_rule(&mut self, rule: RewriteRule) {
        self.rules.push(rule);
    }

    /// 批量添加规则
    pub fn add_rules(&mut self, rules: impl IntoIterator<Item = RewriteRule>) {
        self.rules.extend(rules);
    }

    /// 获取规则数量
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }

    /// 清空规则
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// 重写执行计划
    ///
    /// 对所有注册规则进行迭代应用，直到计划不再变化或达到最大迭代次数
    pub fn rewrite(&self, plan: ExecutionPlan) -> Result<ExecutionPlan, RewriteError> {
        let root = match plan.root {
            Some(ref root) => root.clone(),
            None => return Ok(plan),
        };

        // 创建一个临时的 QueryContext 用于 OptContext
        let temp_qctx = Arc::new(QueryContext::new(Arc::new(crate::api::session::RequestContext::default())));
        let mut ctx = OptContext::new(temp_qctx);
        
        let root_id = 1;
        let new_root = self.rewrite_node(&mut ctx, root, root_id)?;

        let mut new_plan = plan;
        new_plan.set_root(new_root);
        Ok(new_plan)
    }

    /// 重写单个计划节点
    fn rewrite_node(
        &self,
        ctx: &mut OptContext,
        node: PlanNodeEnum,
        node_id: usize,
    ) -> Result<PlanNodeEnum, RewriteError> {
        // 先递归重写子节点
        let node = self.rewrite_children(ctx, node)?;

        // 创建 OptGroupNode
        let group_node = Rc::new(RefCell::new(OptGroupNode::new(node_id, node)));
        ctx.add_group_node(group_node.clone())?;

        // 迭代应用规则直到收敛
        let mut current_node = group_node.borrow().plan_node.clone();
        let mut changed = true;
        let mut iterations = 0;

        while changed && iterations < self.max_iterations {
            changed = false;
            iterations += 1;

            for rule in &self.rules {
                // 检查规则是否匹配
                if rule.matches(ctx, &group_node)? {
                    // 应用规则
                    if let Some(result) = rule.apply(ctx, &group_node)? {
                        if !result.new_group_nodes.is_empty() {
                            current_node = result.new_group_nodes[0].borrow().plan_node.clone();
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(current_node)
    }

    /// 递归重写子节点
    fn rewrite_children(
        &self,
        ctx: &mut OptContext,
        mut node: PlanNodeEnum,
    ) -> Result<PlanNodeEnum, RewriteError> {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        match &mut node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Project(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Aggregate(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Sort(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Limit(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::TopN(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            PlanNodeEnum::Dedup(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, input_node, node_id)?;
                n.set_input(new_input);
            }
            // 双输入节点（连接）
            PlanNodeEnum::HashInnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::CrossJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::InnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::LeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, left, left_id)?;
                let new_right = self.rewrite_node(ctx, right, right_id)?;
                n.set_left_input(new_left);
                n.set_right_input(new_right);
            }
            // Union 节点
            PlanNodeEnum::Union(n) => {
                let deps: Vec<PlanNodeEnum> = n.dependencies()
                    .iter()
                    .map(|dep| dep.clone_plan_node())
                    .collect();
                let mut _new_deps = Vec::new();
                for dep in deps.into_iter() {
                    let node_id = ctx.allocate_node_id();
                    let new_dep = self.rewrite_node(ctx, dep, node_id)?;
                    _new_deps.push(new_dep);
                }
                // TODO: UnionNode 需要支持设置新的 dependencies
            }
            // 叶子节点无需处理
            _ => {}
        }

        Ok(node)
    }
}

impl Default for PlanRewriter {
    fn default() -> Self {
        Self::from_registry(RuleRegistry::default())
    }
}

use crate::query::planner::plan::ExecutionPlan;

/// 创建默认的计划重写器
///
/// 包含所有标准的启发式重写规则，使用静态分发。
pub fn create_default_rewriter() -> PlanRewriter {
    PlanRewriter::default()
}

/// 重写执行计划的便捷函数
pub fn rewrite_plan(plan: ExecutionPlan) -> Result<ExecutionPlan, RewriteError> {
    let rewriter = create_default_rewriter();
    rewriter.rewrite(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_rewriter_default() {
        let rewriter = PlanRewriter::default();
        assert_eq!(rewriter.rule_count(), 35); // 7 + 7 + 12 + 2 + 6 + 1 = 35
    }

    #[test]
    fn test_plan_rewriter_new() {
        let rewriter = PlanRewriter::new();
        assert_eq!(rewriter.rule_count(), 0);
    }

    #[test]
    fn test_plan_rewriter_add_rule() {
        let mut rewriter = PlanRewriter::new();
        assert_eq!(rewriter.rule_count(), 0);
        
        use crate::query::planner::rewrite::elimination::EliminateFilterRule;
        rewriter.add_rule(RewriteRule::EliminateFilter(EliminateFilterRule));
        
        assert_eq!(rewriter.rule_count(), 1);
    }
}
