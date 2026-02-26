//! 计划重写器实现
//!
//! 管理所有启发式重写规则，按顺序应用到计划树。
//! 使用静态分发（枚举）替代动态分发，提供更好的性能。
//!
//! # 性能优势
//!
//! - 无动态分发开销（无虚函数表查找）
//! - 无堆分配（规则存储在栈上）
//! - 更好的缓存局部性
//! - 编译器可以内联优化

use crate::query::planner::plan::PlanNodeEnum;
use crate::query::planner::plan::ExecutionPlan;
use crate::query::planner::rewrite::context::RewriteContext;
use crate::query::planner::rewrite::result::RewriteResult;
use crate::query::planner::rewrite::rule_enum::{RewriteRule as RewriteRuleEnum, RuleRegistry};

/// 计划重写器
///
/// 管理所有启发式重写规则，按顺序应用。
/// 规则按添加顺序执行，每个规则可能被多次应用直到不再产生变化。
///
/// 使用静态分发枚举存储规则，避免动态分发的开销。
#[derive(Debug)]
pub struct PlanRewriter {
    /// 已注册的规则列表（静态分发）
    rules: Vec<RewriteRuleEnum>,
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
    pub fn add_rule(&mut self, rule: RewriteRuleEnum) {
        self.rules.push(rule);
    }

    /// 批量添加规则
    pub fn add_rules(&mut self, rules: impl IntoIterator<Item = RewriteRuleEnum>) {
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
    pub fn rewrite(&self, plan: ExecutionPlan) -> RewriteResult<ExecutionPlan> {
        let root = match plan.root {
            Some(ref root) => root.clone(),
            None => return Ok(plan),
        };

        let mut ctx = RewriteContext::new();
        let root_id = ctx.allocate_node_id();
        let new_root = self.rewrite_node(&mut ctx, &root, root_id)?;

        let mut new_plan = plan;
        new_plan.set_root(new_root);
        Ok(new_plan)
    }

    /// 重写单个计划节点
    fn rewrite_node(
        &self,
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
        node_id: usize,
    ) -> RewriteResult<PlanNodeEnum> {
        // 先递归重写子节点
        let node = self.rewrite_children(ctx, node)?;

        // 注册节点到上下文
        ctx.register_node(node_id, node.clone());

        // 迭代应用规则直到收敛
        let mut current_node = node;
        let mut changed = true;
        let mut iterations = 0;

        while changed && iterations < self.max_iterations {
            changed = false;
            iterations += 1;

            for rule in &self.rules {
                // 检查规则是否匹配
                if rule.matches(&current_node) {
                    // 应用规则
                    if let Some(result) = rule.apply(ctx, &current_node)? {
                        if let Some(new_node) = result.first_new_node() {
                            current_node = new_node.clone();
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
        ctx: &mut RewriteContext,
        node: &PlanNodeEnum,
    ) -> RewriteResult<PlanNodeEnum> {
        use crate::query::planner::plan::core::nodes::plan_node_traits::SingleInputNode;

        match node {
            // 单输入节点
            PlanNodeEnum::Filter(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Filter(new_node))
            }
            PlanNodeEnum::Project(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Project(new_node))
            }
            PlanNodeEnum::Aggregate(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Aggregate(new_node))
            }
            PlanNodeEnum::Sort(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Sort(new_node))
            }
            PlanNodeEnum::Limit(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Limit(new_node))
            }
            PlanNodeEnum::TopN(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::TopN(new_node))
            }
            PlanNodeEnum::Dedup(n) => {
                let input_node = n.input().clone_plan_node();
                let node_id = ctx.allocate_node_id();
                let new_input = self.rewrite_node(ctx, &input_node, node_id)?;
                let mut new_node = n.clone();
                new_node.set_input(new_input);
                Ok(PlanNodeEnum::Dedup(new_node))
            }
            // 双输入节点（连接）
            PlanNodeEnum::HashInnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::HashInnerJoin(new_node))
            }
            PlanNodeEnum::HashLeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::HashLeftJoin(new_node))
            }
            PlanNodeEnum::CrossJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::CrossJoin(new_node))
            }
            PlanNodeEnum::InnerJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::InnerJoin(new_node))
            }
            PlanNodeEnum::LeftJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::LeftJoin(new_node))
            }
            PlanNodeEnum::FullOuterJoin(n) => {
                let left = n.left_input().clone_plan_node();
                let right = n.right_input().clone_plan_node();
                let left_id = ctx.allocate_node_id();
                let right_id = ctx.allocate_node_id();
                let new_left = self.rewrite_node(ctx, &left, left_id)?;
                let new_right = self.rewrite_node(ctx, &right, right_id)?;
                let mut new_node = n.clone();
                new_node.set_left_input(new_left);
                new_node.set_right_input(new_right);
                Ok(PlanNodeEnum::FullOuterJoin(new_node))
            }
            // 多输入节点
            PlanNodeEnum::Union(n) => {
                let deps: Vec<PlanNodeEnum> = n.dependencies()
                    .iter()
                    .map(|dep| dep.clone_plan_node())
                    .collect();
                let mut new_deps = Vec::new();
                for dep in deps.iter() {
                    let node_id = ctx.allocate_node_id();
                    let new_dep = self.rewrite_node(ctx, dep, node_id)?;
                    new_deps.push(new_dep);
                }
                let mut new_node = n.clone();
                new_node.set_dependencies(new_deps);
                Ok(PlanNodeEnum::Union(new_node))
            }
            // 叶子节点无需处理
            _ => Ok(node.clone()),
        }
    }
}

impl Default for PlanRewriter {
    fn default() -> Self {
        Self::from_registry(RuleRegistry::default())
    }
}

/// 创建默认的计划重写器
///
/// 包含所有标准的启发式重写规则，使用静态分发。
pub fn create_default_rewriter() -> PlanRewriter {
    PlanRewriter::default()
}

/// 重写执行计划的便捷函数
pub fn rewrite_plan(plan: ExecutionPlan) -> RewriteResult<ExecutionPlan> {
    let rewriter = create_default_rewriter();
    rewriter.rewrite(plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_rewriter_default() {
        let rewriter = PlanRewriter::default();
        assert!(rewriter.rule_count() > 0);
    }

    #[test]
    fn test_plan_rewriter_new() {
        let rewriter = PlanRewriter::new();
        assert_eq!(rewriter.rule_count(), 0);
    }

    #[test]
    fn test_plan_rewriter_add_rule() {
        use crate::query::planner::rewrite::elimination::EliminateFilterRule;
        
        let mut rewriter = PlanRewriter::new();
        assert_eq!(rewriter.rule_count(), 0);
        
        rewriter.add_rule(RewriteRuleEnum::EliminateFilter(EliminateFilterRule));
        
        assert_eq!(rewriter.rule_count(), 1);
    }
}
